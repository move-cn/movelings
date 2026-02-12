use crate::utils::{
    extract_hints_from_exercise, find_exercise_name_from_path, get_exercises,
    load_completed_exercises, save_completed_exercise, show_default_hints, suggest_next_exercise,
};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use notify::{Watcher, RecursiveMode};
use colored::*;

pub fn list_exercises() {
    let exercises = get_exercises();
    let completed = load_completed_exercises();
    
    println!("{}", "📚 Available exercises:".blue());
    
    for (i, exercise) in exercises.iter().enumerate() {
        let icon = if completed.contains(exercise) { "✅".green() } else { "📝".yellow() };
        println!("  {}. {} {} (run '{}' to test)", i + 1, icon, exercise, format!("movelings run {}", exercise).cyan());
    }
    
    println!();
    let progress_bar = format!("📊 Progress: {}/{} completed ({:.1}%)",
             completed.len(),
             exercises.len(),
             (completed.len() as f64 / exercises.len() as f64) * 100.0);
    println!("{}", progress_bar.bold());
    println!("💡 Icons: {} = Completed, {} = Not completed yet", "✅".green(), "📝".yellow());
    println!("💡 To get hints: {}", "movelings hint <exercise_name>".cyan());
}

pub fn check_exercise(name: &str) -> bool {
    let exercise_path = format!("exercises/{}", name);
    if !Path::new(&exercise_path).exists() {
        println!("{} Exercise '{}' not found!", "❌".red(), name.bold());
        println!("Run '{}' to see available exercises.", "movelings list".cyan());
        return false;
    }
    
    println!("{} Checking exercise: {}", "🔍".blue(), name.bold());
    println!("{} Path: {}", "📁".blue(), exercise_path);
    
    println!("{}", "⏳ Running tests... (this may take a moment)".dimmed());
    
    let output = Command::new("sui")
        .args(&["move", "test"])
        .current_dir(&exercise_path)
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("{} Exercise '{}' completed successfully!", "✅".green(), name.bold());
                
                save_completed_exercise(name);
                
                println!("{}", "🎉 Great job! Try the next exercise.".green());
                suggest_next_exercise(name);
                true
            } else {
                println!("{} Exercise '{}' failed:", "❌".red(), name.bold());
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                if !stderr.is_empty() {
                    println!("\n{}", "📋 Error output:".red());
                    println!("{}", stderr.red());
                }
                if !stdout.is_empty() {
                    println!("\n{}", "📋 Test output:".yellow());
                    println!("{}", stdout.yellow());
                }
                
                show_hint(name);
                false
            }
        }
        Err(e) => {
            println!("{} Failed to run sui command: {}", "❌".red(), e);
            println!("💡 Make sure Sui CLI is installed and in your PATH");
            println!("💡 You can also try running 'sui move test' manually in the exercise directory");
            false
        }
    }
}

pub fn show_hint(exercise: &str) -> bool {
    println!();
    println!("{} Hints for exercise '{}':", "💡".yellow(), exercise.bold());

    let hints = extract_hints_from_exercise(exercise);

    if hints.is_empty() {
        show_default_hints();
    } else {
        for (i, hint) in hints.iter().enumerate() {
            println!("  {}. {}", i + 1, hint.italic());
        }
    }

    println!();
    println!("{} Exercise file location:", "📝".blue());
    println!("   {}", format!("exercises/{}/sources/", exercise).cyan());
    true
}

pub fn reset_progress() {
    if Path::new(".movelings_progress").exists() {
        match std::fs::remove_file(".movelings_progress") {
            Ok(_) => println!("{}", "🔄 Progress reset successfully!".green()),
            Err(e) => println!("{} Failed to reset progress: {}", "❌".red(), e),
        }
    } else {
        println!("{}", "ℹ️  No progress file found, nothing to reset.".yellow());
    }
}

pub fn show_progress() {
    let exercises = get_exercises();
    let completed = load_completed_exercises();
    
    println!("{}", "📊 Detailed Progress Report".bold().blue());
    println!("==========================");
    println!();
    
    if exercises.is_empty() {
        println!("{}", "No exercises found!".yellow());
        return;
    }
    
    println!("{}", "📚 Exercise Status:".blue());
    for (i, exercise) in exercises.iter().enumerate() {
        let status = if completed.contains(exercise) {
            "✅ Completed".green()
        } else {
            "📝 Not completed".yellow()
        };
        println!("  {}. {} - {}", i + 1, exercise, status);
    }
    
    println!();
    println!("{}", "📈 Statistics:".blue());
    println!("  Total exercises: {}", exercises.len());
    println!("  Completed: {}", completed.len());
    println!("  Remaining: {}", exercises.len() - completed.len());
    println!("  Progress: {:.1}%", (completed.len() as f64 / exercises.len() as f64) * 100.0);
    
    if completed.len() == exercises.len() {
        println!();
        println!("{}", "🎉 Congratulations! You've completed all exercises!".green().bold());
    } else if let Some(next) = exercises.iter().find(|ex| !completed.contains(*ex)) {
        println!();
        println!("➡️  Next exercise to try: {}", format!("movelings run {}", next).cyan());
    }
}

pub fn watch_mode() {
    println!("{}", "👀 Entering watch mode. Press Ctrl+C to exit.".bold().yellow());
    println!("{}", "📂 Watching for changes in the 'exercises' directory...".dimmed());

    let (tx, rx) = channel();

    let mut watcher = match notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            if let Err(e) = tx.send(event) {
                eprintln!("{} {}", "❌ Error sending event:".red(), e);
            }
        } else if let Err(e) = res {
            eprintln!("{} {}", "❌ Watch error:".red(), e);
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{} {}", "❌ Failed to create watcher:".red(), e);
            return;
        }
    };

    if let Err(e) = watcher.watch(Path::new("exercises"), RecursiveMode::Recursive) {
        eprintln!("{} {}", "❌ Failed to start watching 'exercises' directory:".red(), e);
        return;
    }

    let exercises = get_exercises();
    let completed = load_completed_exercises();
    if let Some(first_exercise) = exercises.iter().find(|e| !completed.contains(*e)) {
        println!("\n{} Starting with the first uncompleted exercise: {}", "🚀".blue(), first_exercise.bold());
        check_exercise(first_exercise);
        println!("{}", "----------------------------------------------------".dimmed());
        println!("{}", "👀 Watching for changes... Press Ctrl+C to exit.".yellow());
    }


    loop {
        match rx.recv() {
            Ok(event) => {
                if let notify::Event {
                    kind: notify::EventKind::Modify(_),
                    paths,
                    ..
                } = event
                {
                    for path in paths {
                        let is_in_sources = path.components().any(|c| c.as_os_str() == "sources");
                        let is_in_build = path.components().any(|c| c.as_os_str() == "build");
                        let is_move_file = path.extension().map_or(false, |s| s == "move");

                        if is_move_file && is_in_sources && !is_in_build {
                            if let Some(exercise_name) = find_exercise_name_from_path(&path) {
                                print!("\x1B[2J\x1B[1;1H");
                                println!("{}", "----------------------------------------------------".dimmed());
                                println!("{} Detected change in: {}", "✨".yellow(), path.display());
                                check_exercise(&exercise_name);
                                println!("{}", "----------------------------------------------------".dimmed());
                                println!("{}", "👀 Watching for next change... Press Ctrl+C to exit.".yellow());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{} {}", "❌ Error receiving event:".red(), e);
                break;
            }
        }
    }
}

pub fn show_menu() {
    println!("{}", "🏃‍♂️ Welcome to Movelings!".bold().blue());
    println!("=======================");
    println!();
    println!("💡 To get started, run `{}` to see all exercises.", "movelings list".cyan());
    println!("💡 Or, run `{}` to test a specific exercise.", "movelings run <exercise_name>".cyan());
    println!("💡 Use `{}` to see all available commands.", "movelings --help".cyan());
    println!();
    
    let exercises = get_exercises();
    let completed = load_completed_exercises();
    
    println!("{}", "📚 Quick overview:".blue());
    println!("  Total exercises: {}", exercises.len());
    println!("  Completed: {}", completed.len());
    println!("  Remaining: {}", exercises.len() - completed.len());
    
    if !completed.is_empty() {
        println!("  Progress: {:.1}%", (completed.len() as f64 / exercises.len() as f64) * 100.0);
    }
    
    println!();
    if let Some(next) = exercises.iter().find(|ex| !completed.contains(*ex)) {
        println!("🚀 Next up: {}", format!("movelings run {}", next).cyan());
    } else {
        println!("{}", "🎉 Congratulations! You've completed all exercises!".green().bold());
    }
}