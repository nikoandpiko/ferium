use crate::error::{Error, Result};
use ferinth::{
	structures::{project_structs::Project, ProjectType},
	Ferinth,
};
use furse::{structures::mod_structs::Mod, Furse};
use libium::config;
use octocrab::{models::Repository, repos::RepoHandler};

/// Check if repo of `repo_handler` exists and releases mods, and if so add the repo to `profile`
pub async fn github(
	repo_handler: RepoHandler<'_>,
	profile: &mut config::structs::Profile,
) -> Result<Repository> {
	let repo = match repo_handler.get().await {
		Ok(repo) => repo,
		Err(err) => {
			return Err(Error::QuitFormatted(format!(
				"Repository does not exist ({})",
				err
			)))
		},
	};
	// Get the name of the repository as a tuple
	let repo_name_split = repo
		.full_name
		.as_ref()
		.ok_or(Error::OptionError)?
		.split('/')
		.collect::<Vec<_>>();
	let repo_name = (repo_name_split[0].into(), repo_name_split[1].into());

	// Check if repo has already been added
	if profile.github_repos.contains(&repo_name) {
		return Err(Error::Quit("Repository already added to profile"));
	}

	let releases = repo_handler.releases().list().send().await?;
	let mut contains_jar_asset = false;

	// Search every asset to check if the releases contain JAR files (a mod file)
	'outer: for release in releases {
		for asset in release.assets {
			if asset.name.contains("jar") {
				// If JAR release is found, set flag to true and break
				contains_jar_asset = true;
				break 'outer;
			}
		}
	}

	if contains_jar_asset {
		profile.github_repos.push(repo_name);
		Ok(repo)
	} else {
		Err(Error::Quit("Repository does not release mods"))
	}
}

/// Check if `project_id` exists and is a mod, if so add that project ID to `profile`
/// Returns the project struct
pub async fn modrinth(
	modrinth: &Ferinth,
	project_id: String,
	profile: &mut config::structs::Profile,
) -> Result<Project> {
	match modrinth.get_project(&project_id).await {
		Ok(project) => {
			// Check if project has already been added
			if profile.modrinth_mods.contains(&project.id) {
				Err(Error::Quit("Mod already added to profile"))
			// Check that the project is a mod
			} else if project.project_type != ProjectType::Mod {
				Err(Error::Quit("Project is not a mod"))
			} else {
				profile.modrinth_mods.push(project.id.clone());
				Ok(project)
			}
		},
		Err(err) => Err(Error::QuitFormatted(format!(
			"Project does not exist ({})",
			err
		))),
	}
}

/// Check if `project_id` exists, if so add that mod to `profile`
/// Returns the mod struct
pub async fn curseforge(
	curseforge: &Furse,
	project_id: i32,
	profile: &mut config::structs::Profile,
) -> Result<Mod> {
	match curseforge.get_mod(project_id).await {
		Ok(mod_) => {
			// Check if project has already been added
			if profile.curse_projects.contains(&mod_.id) {
				Err(Error::Quit("Project already added to profile"))
			} else {
				profile.curse_projects.push(mod_.id);
				Ok(mod_)
			}
		},
		Err(err) => Err(Error::QuitFormatted(format!(
			"Project does not exist or is not a mod ({})",
			err
		))),
	}
}