use chapchap_common::rule_manager::{
    program_monitor::{self, Action, Filter},
    Module, Rule,
};

mod rule_manager;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let (ui_worker, _) = ui::init()?;
    let rule_manager = rule_manager::start().await?;

    ////let rules = rule_manager.rules().await?;
    ////println!("rules: {rules:?}");

    for i in 100..110 {
        let rule = Rule {
            name: format!("Rule {i}"),
            is_active: true,
            module: Module::ProgramMonitor(program_monitor::Rule {
                filter: Filter::Basic,
                action: Action::BlockProgramExecution(i),
            }),
        };

        let rule_id = rule_manager.add_rule(rule).await?;
        println!("rule_id: {rule_id:?}");

        //std::thread::sleep(std::time::Duration::from_secs(1));

        //let rules = rule_manager.rules().await?;
        //println!("rules: {rules:?}");
    }

    //ui_worker.join().unwrap();
    Ok(())
}
