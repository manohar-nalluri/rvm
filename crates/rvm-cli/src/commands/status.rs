use rvm_core::Workspace;
use rvm_tracker::status;

pub fn execute() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let current = ws.current_branch()?;

    let dashboard = status::build_dashboard(&ws.branches_dir())?;

    println!("RVM Status Dashboard");
    println!("====================");
    println!("Current branch: {}", current);
    println!(
        "Total: {} applications ({} active)\n",
        dashboard.total, dashboard.active
    );

    let status_order = [
        "Interviewing",
        "Applied",
        "Screening",
        "Offered",
        "Draft",
        "Rejected",
        "Ghosted",
        "Withdrawn",
        "Accepted",
    ];

    for status_name in &status_order {
        if let Some(branches) = dashboard.by_status.get(*status_name) {
            println!("  {} ({}):", status_name, branches.len());
            for b in branches {
                let deadline_flag = if b.deadline_passed { " [OVERDUE]" } else { "" };
                println!(
                    "    {} — {} @ {} ({}d ago){}",
                    b.branch_name, b.role, b.company, b.days_since_activity, deadline_flag
                );
            }
            println!();
        }
    }

    Ok(())
}
