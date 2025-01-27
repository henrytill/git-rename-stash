use std::{env, process};

use git2::{Commit, Error, Repository};

fn rename_stash(repo: &Repository, stash_index: usize, new_message: &str) -> Result<(), Error> {
    // Read the stash reflog
    let mut reflog = repo.reflog("refs/stash")?;
    let max_index = reflog.len();

    if stash_index >= max_index {
        return Err(Error::from_str(&format!(
            "Invalid stash index: {} (max: {})",
            stash_index,
            max_index.saturating_sub(1)
        )));
    }

    // Get the target stash entry
    let target_entry = reflog
        .get(stash_index)
        .ok_or_else(|| Error::from_str("Failed to get stash entry"))?;

    let old_stash_oid = target_entry.id_new();

    // Get the original stash commit
    let stash_commit = repo.find_commit(old_stash_oid)?;

    // Get the tree from the original commit
    let tree = stash_commit.tree()?;

    // Get the original author & committer
    let author = stash_commit.author();
    let committer = stash_commit.committer();

    // Get the parents from the original commit
    let parent_commits: Vec<Commit> = stash_commit.parents().collect();
    let parent_refs: Vec<&Commit> = parent_commits.iter().collect();

    // Create a new commit with the new message but same tree and parents
    let new_stash_oid = repo.commit(
        None,       // Don't update any references
        &author,    // Original author
        &committer, // Original committer
        new_message,
        &tree,
        &parent_refs,
    )?;

    // First remove the old entry
    reflog.remove(stash_index, false)?;
    // Then add the new entry at the same position
    reflog.append(new_stash_oid, &author, Some(new_message))?;
    reflog.write()?;

    Ok(())
}

fn print_usage(program_name: &str) {
    println!("Usage: {} <stash-index> <new-message>", program_name);
    println!("Example: {} 0 \"New stash message\"", program_name);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage(&args[0]);
        process::exit(1);
    }

    // Parse stash index
    let stash_index = match args[1].parse::<usize>() {
        Ok(index) => index,
        Err(_) => {
            eprintln!("Invalid stash index: {}", args[1]);
            process::exit(1);
        }
    };

    let new_message = args[2].trim();
    if new_message.is_empty() {
        eprintln!("Stash message cannot be empty");
        process::exit(1);
    }

    // Open the repository
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Failed to open repository: {}", e);
            process::exit(1);
        }
    };

    // Rename the stash
    match rename_stash(&repo, stash_index, new_message) {
        Ok(()) => println!("Successfully renamed stash {}", stash_index),
        Err(e) => {
            eprintln!("Error renaming stash: {}", e);
            process::exit(1);
        }
    }
}
