use colored::Colorize;
use eyre::Result;
use jju_core::stack_sync::StackSyncPlan;
use std::io::Write;

pub(super) fn confirm_plan(plan: &StackSyncPlan) -> Result<bool> {
    println!(
        "Will rebase the following commits on top of {}:",
        plan.trunk.cyan()
    );

    for root in &plan.roots {
        println!(
            "  {}  {}",
            root.change_id.purple(),
            root.description.dimmed()
        );
        println!(
            "  {}",
            format!(
                "jj rebase --source (-s) {} --onto (-o) {} --skip-emptied",
                root.change_id, plan.trunk
            )
            .dimmed()
        );
    }

    print!("Continue? [y/N] ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}
