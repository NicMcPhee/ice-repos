// TODO: Clean up the logging elsewhere to use `gloo::console::log`.
use gloo::console::log;

use crate::repository::Repository;

pub fn archive_repositories<'a>(repos: impl Iterator<Item = &'a Repository>) {
    for repo in repos {
        // TODO: We need to change this to actually make the REST request
        //  to the GitHub servers.
        log!(format!("We are archiving {}.", repo.name));
    }
}
