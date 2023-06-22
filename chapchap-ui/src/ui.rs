use anyhow::anyhow;
use futures::FutureExt;
use slint::ComponentHandle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod generated_code {
    slint::include_modules!();
}
use generated_code::MainWindow;

pub fn init() -> anyhow::Result<(Worker, MainWindow)> {
    let main_window = MainWindow::new()?;
    let worker = Worker::new(&main_window);

    main_window.run()?;
    Ok((worker, main_window))
}

enum Message {
    Quit,
}

pub struct Worker {
    channel: UnboundedSender<Message>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(ui: &MainWindow) -> Self {
        let (channel, r) = tokio::sync::mpsc::unbounded_channel();

        let worker_thread = std::thread::spawn({
            let handle_weak = ui.as_weak();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(worker_loop(r, handle_weak))
                    .unwrap()
            }
        });

        Self {
            channel,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(Message::Quit);
        self.worker_thread.join()
    }
}

async fn worker_loop(
    mut r: UnboundedReceiver<Message>,
    handle: slint::Weak<MainWindow>,
) -> anyhow::Result<()> {
    handle
        .upgrade_in_event_loop(|h| {
            h.on_open_add_rule_dialog(|| open_select_file_dialog().into());
        })
        .map_err(|_| anyhow!("settign up callbacks"))?;

    loop {
        let message = tokio::select! {
            m = r.recv().fuse() => {
                match m {
                    None => return Ok(()),
                    Some(m) => m,
                }
            }
        };

        match message {
            Message::Quit => return Ok(()),
        }
    }
}

fn open_select_file_dialog() -> String {
    let dialog = rfd::FileDialog::new().set_title("Select program file");
    dialog
        .pick_file()
        .map(|p| p.display().to_string())
        .unwrap_or("".into())
}
