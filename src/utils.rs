use std::fs;
use std::collections::HashSet;
use std::path::Path;
use colored::*;

pub fn get_exercises() -> Vec<String> {
    let mut exercises = Vec::new();
    if let Ok(entries) = fs::read_dir("exercises") {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    exercises.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }
    exercises.sort();
    exercises
}

pub fn load_completed_exercises() -> HashSet<String> {
    let mut completed = HashSet::new();
    if let Ok(content) = fs::read_to_string(".movelings_progress") {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                completed.insert(line.to_string());
            }
        }
    }
    completed
}

pub fn save_completed_exercise(exercise: &str) {
    let mut completed = load_completed_exercises();
    if completed.insert(exercise.to_string()) {
        save_progress(&completed);
        println!("{}", "💾 Progress saved!".green());
    }
}

pub fn save_progress(completed: &HashSet<String>) {
    let mut exercises: Vec<String> = completed.iter().cloned().collect();
    exercises.sort();
    
    if let Err(e) = fs::write(".movelings_progress", exercises.join("\n")) {
        eprintln!("⚠️  Warning: Failed to save progress: {}", e);
    }
}

pub fn suggest_next_exercise(current: &str) {
    let exercises = get_exercises();
    let completed = load_completed_exercises();
    
    if let Some(pos) = exercises.iter().position(|x| x == current) {
        if pos + 1 < exercises.len() {
            println!("➡️  Next exercise: {}", format!("movelings run {}", exercises[pos + 1]).cyan());
        } else {
            println!("{}", "🎊 Congratulations! You've completed all exercises!".green().bold());
            println!("🏆 Final progress: {}/{} exercises completed!", completed.len(), exercises.len());
        }
    }
}

pub fn extract_hints_from_exercise(exercise: &str) -> Vec<String> {
    let mut hints = Vec::new();
    let sources_path = format!("exercises/{}/sources", exercise);
    
    if let Ok(entries) = fs::read_dir(&sources_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(ext) = entry.path().extension() {
                    if ext == "move" {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            hints.extend(extract_hints_from_content(&content));
                        }
                    }
                }
            }
        }
    }
    
    hints
}

pub fn extract_hints_from_content(content: &str) -> Vec<String> {
    let mut hints = Vec::new();
    let mut seen = std::collections::HashSet::new();
    
    for line in content.lines() {
        let line = line.trim();
        let mut hint_text = None;
        
        if line.starts_with("// HINT:") || line.starts_with("/// HINT:") {
            let hint = line.trim_start_matches("// HINT:")
                          .trim_start_matches("/// HINT:")
                          .trim();
            if !hint.is_empty() {
                hint_text = Some(hint.to_string());
            }
        }
        
        else if line.starts_with("// TODO:") || line.starts_with("/// TODO:") {
            let todo = line.trim_start_matches("// TODO:")
                          .trim_start_matches("/// TODO:")
                          .trim();
            if !todo.is_empty() {
                hint_text = Some(format!("TODO: {}", todo));
            }
        }
        
        else if line.starts_with("//") {
            let comment = line.trim_start_matches("//").trim();
            if comment.len() > 2 && 
               comment.chars().nth(0).unwrap_or(' ').is_ascii_digit() && 
               comment.chars().nth(1) == Some('.') {
                hint_text = Some(comment.to_string());
            }
        }
        
        if let Some(hint) = hint_text {
            if seen.insert(hint.clone()) {
                hints.push(hint);
            }
        }
    }
    
    hints
}

pub fn show_default_hints() {
    let hints = vec![
        "Read the exercise file comments carefully",
        "Look for TODO markers that guide you",
        "Check the test functions to understand what's expected",
        "Try running 'sui move test' manually to see detailed errors",
    ];
    
    for (i, hint) in hints.iter().enumerate() {
        println!("  {}. {}", i + 1, hint.italic());
    }
}

pub fn find_exercise_name_from_path(path: &Path) -> Option<String> {
    path.ancestors()
        .find(|p| {
            p.parent().map_or(false, |parent| parent.file_name() == Some("exercises".as_ref()))
        })
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}
pub fn find_exercise_by_prefix(prefix: &str) -> Result<String, String> {
    let exercises = get_exercises();
    let mut matches: Vec<_> = exercises
        .iter()
        .filter(|e| e.starts_with(prefix))
        .collect();

    match matches.len() {
        1 => Ok(matches.remove(0).clone()),
        0 => Err(format!("No exercise found with prefix '{}'", prefix)),
        _ => Err(format!(
            "Multiple exercises found with prefix '{}': {:?}",
            prefix, matches
        )),
    }
}