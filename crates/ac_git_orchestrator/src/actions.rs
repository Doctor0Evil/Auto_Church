use ac_aln_rt::{
    errors::AlnError,
    exec::{json_ok, run_shell, session_key_from_template, update_state},
    model::{CloneOptions, GitDiffType, HistoryAction, Scope, SubmoduleAction, P4Action},
    session::Session,
};
use serde_json::Value;

use crate::config::git_script_config;
use crate::session_store::SessionStore;

pub struct GitActions {
    redis_url: String,
}

impl GitActions {
    pub fn new(redis_url: &str) -> Self {
        Self {
            redis_url: redis_url.to_string(),
        }
    }

    async fn get_or_create_session(
        &self,
        user_id: &str,
        state: &str,
    ) -> Result<(SessionStore, Session, String), AlnError> {
        let config = git_script_config();
        let key = session_key_from_template(&config.session_key_template, user_id);
        let mut store = SessionStore::new(&self.redis_url).await?;
        if let Some(session) = store.get(&key).await? {
            Ok((store, session, key))
        } else {
            let session = Session::new(user_id.to_string(), config.bot_id.clone(), state);
            Ok((store, session, key))
        }
    }

    pub async fn config_list(
        &self,
        user_id: &str,
        scope: Scope,
    ) -> Result<Value, AlnError> {
        let (mut store, mut session, key) =
            self.get_or_create_session(user_id, "config_list").await?;

        let mut outputs = Vec::new();

        match scope {
            Scope::All => {
                outputs.push(run_shell("git config --list --show-origin").await?);
                outputs.push(run_shell("git config --list --show-scope").await?);
            }
            Scope::System => {
                outputs.push(run_shell("git config --list --system").await?);
            }
            Scope::Global => {
                outputs.push(run_shell("git config --list --global").await?);
            }
            Scope::Local => {
                outputs.push(run_shell("git config --list --local").await?);
            }
        }

        update_state(&mut session, "config_list_done");
        store.set(&key, &session).await?;
        Ok(json_ok(
            "executed",
            serde_json::json!({ "outputs": outputs }),
        ))
    }

    pub async fn config_difftool(
        &self,
        user_id: &str,
        tool: &str,
    ) -> Result<Value, AlnError> {
        let (mut store, mut session, key) =
            self.get_or_create_session(user_id, "config_difftool").await?;

        match tool {
            "araxis" => {
                run_shell("git config --global difftool.araxis.path 'C:/Program Files/Araxis/Araxis Merge/compare.exe'").await?;
                run_shell("git config --global mergetool.araxis.path 'C:/Program Files/Araxis/Araxis Merge/compare.exe'").await?;
            }
            "beyondcompare" => {
                run_shell("git config --global difftool.beyondcompare.path 'C:/Program Files/Beyond Compare 4/bcomp.exe'").await?;
                run_shell("git config --global mergetool.beyondcompare.path 'C:/Program Files/Beyond Compare 4/bcomp.exe'").await?;
            }
            "difftastic" => {
                run_shell("git config --global difftool.difftastic.cmd 'difft.exe $LOCAL $REMOTE'").await?;
            }
            _ => return Err(AlnError::InvalidInput("unknown tool".into())),
        }

        run_shell("git config --global difftool.prompt false").await?;
        run_shell("git config --global pager.difftool true").await?;

        update_state(&mut session, "config_difftool_done");
        store.set(&key, &session).await?;
        Ok(json_ok(
            "configured",
            serde_json::json!({ "tool": tool }),
        ))
    }

    pub async fn clone_repository(
        &self,
        user_id: &str,
        repo_url: &str,
        options: CloneOptions,
    ) -> Result<Value, AlnError> {
        let (mut store, mut session, key) =
            self.get_or_create_session(user_id, "clone_repository").await?;

        let mut cmd = String::from("git clone");

        if !options.autocrlf {
            cmd.push_str(" --config core.autocrlf=false");
        }
        if let Some(depth) = options.depth {
            cmd.push_str(&format!(" --depth {}", depth));
        }
        if options.single_branch {
            cmd.push_str(" --single-branch");
        }
        cmd.push(' ');
        cmd.push_str(repo_url);

        let output = run_shell(&cmd).await?;

        update_state(&mut session, "clone_repository_done");
        store.set(&key, &session).await?;
        Ok(json_ok(
            "cloned",
            serde_json::json!({ "repo": repo_url, "output": output }),
        ))
    }

    pub async fn submodule_management(
        &self,
        user_id: &str,
        action: SubmoduleAction,
    ) -> Result<Value, AlnError> {
        let (mut store, mut session, key) =
            self.get_or_create_session(user_id, "submodule_management").await?;

        let mut logs = Vec::new();

        match action {
            SubmoduleAction::Init => {
                logs.push(run_shell("git submodule update --init --recursive").await?);
            }
            SubmoduleAction::Sync => {
                logs.push(
                    run_shell(
                        "git submodule sync --recursive && git submodule update --init --recursive",
                    )
                    .await?,
                );
            }
            SubmoduleAction::Add {
                repo_url,
                path,
                branch,
                depth,
            } => {
                let mut cmd = String::from("git submodule add");
                if let Some(b) = branch.clone() {
                    cmd.push_str(&format!(" -b {}", b));
                }
                if let Some(d) = depth {
                    cmd.push_str(&format!(" --depth {}", d));
                }
                cmd.push(' ');
                cmd.push_str(&repo_url);
                cmd.push(' ');
                cmd.push_str(&path);
                logs.push(run_shell(&cmd).await?);

                if depth.is_some() {
                    let cfg = format!(
                        "git config -f .gitmodules submodule.{}.shallow true",
                        path
                    );
                    logs.push(run_shell(&cfg).await?);
                }
                if let Some(b) = branch {
                    let cfg = format!(
                        "git config -f .gitmodules submodule.{}.branch {}",
                        path, b
                    );
                    logs.push(run_shell(&cfg).await?);
                }
            }
            SubmoduleAction::SetBranch { path, branch } => {
                let cmd = format!("git submodule set-branch -b {} -- {}", branch, path);
                logs.push(run_shell(&cmd).await?);
            }
            SubmoduleAction::Move { old_path, new_path } => {
                let cmd = format!("git mv {} {}", old_path, new_path);
                logs.push(run_shell(&cmd).await?);
                session.data.insert(
                    "old_path".to_string(),
                    serde_json::Value::String(old_path),
                );
            }
            SubmoduleAction::Remove { path } => {
                let cmd = format!("git rm {} && git commit -m 'Remove submodule {}'", path, path);
                logs.push(run_shell(&cmd).await?);
            }
            SubmoduleAction::Deinit { path } => {
                let cmd1 = format!("git submodule deinit -f {}", path);
                let cmd2 = format!("rm -rf .git/modules/{}", path);
                let cmd3 = format!("git rm -f {}", path);
                logs.push(run_shell(&cmd1).await?);
                logs.push(run_shell(&cmd2).await?);
                logs.push(run_shell(&cmd3).await?);
            }
        }

        update_state(&mut session, "submodule_management_done");
        store.set(&key, &session).await?;
        Ok(json_ok(
            "executed",
            serde_json::json!({ "logs": logs }),
        ))
    }

    pub async fn diff_operations(
        &self,
        user_id: &str,
        diff_type: GitDiffType,
        target: Option<String>,
        path: Option<String>,
    ) -> Result<Value, AlnError> {
        let (mut store, mut session, key) =
            self.get_or_create_session(user_id, "diff_operations").await?;

        let cmd = match diff_type {
            GitDiffType::WorkingTree => "git difftool --dir-diff HEAD --".to_string(),
            GitDiffType::Staged => "git difftool --dir-diff --staged".to_string(),
            GitDiffType::Branch => {
                let t = target.ok_or_else(|| AlnError::InvalidInput("target required".into()))?;
                let p = path.unwrap_or_else(|| ".".into());
                format!("git difftool {} -- {}", t, p)
            }
            GitDiffType::Folder => {
                let t = target.ok_or_else(|| AlnError::InvalidInput("target required".into()))?;
                format!("git difftool --dir-diff {}", t)
            }
        };

        let output = run_shell(&cmd).await?;

        update_state(&mut session, "diff_operations_done");
        store.set(&key, &session).await?;
        Ok(json_ok(
            "diff_completed",
            serde_json::json!({ "output": output }),
        ))
    }

    pub async fn history_manipulation(
        &self,
        user_id: &str,
        action: HistoryAction,
    ) -> Result<Value, AlnError> {
        let (mut store, mut session, key) =
            self.get_or_create_session(user_id, "history_manipulation").await?;

        let cmd = match action {
            HistoryAction::UndoCommit => "git reset --soft HEAD^".to_string(),
            HistoryAction::Clean => "git clean -fdx".to_string(),
            HistoryAction::CreatePatch => {
                "git format-patch origin/master --stdout > mypatch.patch".to_string()
            }
            HistoryAction::Squash => "git rebase -i HEAD~2".to_string(),
            HistoryAction::Rebase { target } => format!("git rebase -Xtheirs {}", target),
        };

        let output = run_shell(&cmd).await?;

        update_state(&mut session, "history_manipulation_done");
        store.set(&key, &session).await?;
        Ok(json_ok(
            "executed",
            serde_json::json!({ "output": output }),
        ))
    }

    pub async fn p4_operations(
        &self,
        user_id: &str,
        action: P4Action,
    ) -> Result<Value, AlnError> {
        let (mut store, mut session, key) =
            self.get_or_create_session(user_id, "p4_operations").await?;

        let mut logs = Vec::new();

        logs.push(
            run_shell("git config --global git-p4.skipSubmitEdit true").await?,
        );
        logs.push(
            run_shell("git config --global git-p4.useclientspec true").await?,
        );

        match action {
            P4Action::Clone { depot_path } => {
                let cmd = format!("git p4 clone --detect-branches {}", depot_path);
                logs.push(run_shell(&cmd).await?);
            }
            P4Action::Submit => {
                logs.push(run_shell("git p4 submit").await?);
            }
        }

        update_state(&mut session, "p4_operations_done");
        store.set(&key, &session).await?;
        Ok(json_ok(
            "executed",
            serde_json::json!({ "logs": logs }),
        ))
    }
}
