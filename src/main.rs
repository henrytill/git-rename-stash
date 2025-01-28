use std::{env, process};

use git2::{Error, Repository};

fn rename_stash(repo: &mut Repository, stash_index: usize, new_message: &str) -> Result<(), Error> {
    // Read the stash reflog
    let reflog = repo.reflog("refs/stash")?;
    let max_index = reflog.len();

    if stash_index >= max_index {
        return Err(Error::from_str(&format!(
            "Invalid stash index: {} (max: {})",
            stash_index,
            max_index.saturating_sub(1)
        )));
    }

    // Get and amend the commit
    let new_commit_id = {
        let target_entry = reflog
            .get(stash_index)
            .ok_or_else(|| Error::from_str("Failed to get stash entry"))?;
        let stash_id = target_entry.id_new();
        let commit = repo.find_commit(stash_id)?;
        commit.amend(None, None, None, None, Some(new_message), None)?
    };

    // Remove the old entry
    repo.stash_drop(stash_index)?;

    // Update the stash ref to point to the new commit
    repo.reference("refs/stash", new_commit_id, true, new_message)?;

    Ok(())
}

fn print_usage(program_name: &str) {
    println!("Usage: {} <stash-index> <new-message>", program_name);
    println!("Example: {} 0 \"New stash message\"", program_name);
}

fn parse_stash_ref(stash_ref: &str) -> Result<usize, String> {
    // Match stash@{N} format
    if let Some(index) = stash_ref
        .strip_prefix("stash@{")
        .and_then(|s| s.strip_suffix("}"))
        .and_then(|index_str| index_str.parse::<usize>().ok())
    {
        return Ok(index);
    }

    // Try parsing as plain number
    if let Ok(index) = stash_ref.parse::<usize>() {
        return Ok(index);
    }

    Err(format!("Invalid stash reference: {}", stash_ref))
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage(&args[0]);
        process::exit(1);
    }

    // Parse stash index
    let stash_index = match parse_stash_ref(&args[1]) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let new_message = args[2].trim();
    if new_message.is_empty() {
        eprintln!("Stash message cannot be empty");
        process::exit(1);
    }

    // Open the repository
    let mut repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Failed to open repository: {}", e);
            process::exit(1);
        }
    };

    // Rename the stash
    match rename_stash(&mut repo, stash_index, new_message) {
        Ok(()) => println!("Successfully renamed stash {}", stash_index),
        Err(e) => {
            eprintln!("Error renaming stash: {}", e);
            process::exit(1);
        }
    }
}
