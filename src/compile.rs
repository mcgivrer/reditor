//! Compilation de projet, indépendante du langage.
//!
//! [`CompilationModule`] définit le contrat commun (détection d'un type de
//! projet dans un dossier, puis compilation via un [`Jdk`] ou tout autre
//! outil futur). [`JavaModule`] en est la seule implémentation à ce stade :
//! elle détecte la présence de fichiers `*.java` et délègue la compilation
//! à `javac`.

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Un JDK détecté sur le système, utilisable pour compiler un projet Java.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Jdk {
    pub label: String,
    pub javac: PathBuf,
}

/// Résultat de l'exécution d'un module de compilation.
#[derive(Debug)]
pub struct CompileOutcome {
    pub success: bool,
    pub output: String,
}

/// Contrat commun à tout module de compilation, indépendant du langage
/// concret qu'il prend en charge.
pub trait CompilationModule {
    fn name(&self) -> &'static str;

    /// Vrai si le dossier `root` correspond à un projet que ce module sait
    /// compiler.
    fn detect(&self, root: &Path) -> bool;

    /// Compile le projet situé dans `root` en utilisant le JDK sélectionné.
    fn compile(&self, root: &Path, jdk: &Jdk) -> CompileOutcome;
}

/// Module de compilation pour les projets Java : détecte la présence de
/// fichiers `*.java` dans l'arborescence et compile nativement via `javac`.
pub struct JavaModule;

impl CompilationModule for JavaModule {
    fn name(&self) -> &'static str {
        "Java"
    }

    fn detect(&self, root: &Path) -> bool {
        !find_java_files(root).is_empty()
    }

    fn compile(&self, root: &Path, jdk: &Jdk) -> CompileOutcome {
        let files = find_java_files(root);
        if files.is_empty() {
            return CompileOutcome {
                success: false,
                output: "Aucun fichier .java trouvé dans le projet".to_string(),
            };
        }
        match Command::new(&jdk.javac).args(&files).current_dir(root).output() {
            Ok(output) => {
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&output.stderr));
                CompileOutcome { success: output.status.success(), output: text }
            }
            Err(e) => CompileOutcome {
                success: false,
                output: format!("Erreur au lancement de {} : {e}", jdk.javac.display()),
            },
        }
    }
}

/// Parcourt récursivement `root` à la recherche de fichiers `*.java`.
fn find_java_files(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read_dir.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(OsStr::to_str) == Some("java") {
                result.push(path);
            }
        }
    }
    result
}

/// Modules de compilation disponibles, dans l'ordre où leur détection est
/// tentée.
fn available_modules() -> Vec<Box<dyn CompilationModule>> {
    vec![Box::new(JavaModule)]
}

/// Trouve le premier module de compilation qui reconnaît le projet ouvert
/// dans `root`, s'il en existe un.
pub fn detect_module(root: &Path) -> Option<Box<dyn CompilationModule>> {
    available_modules().into_iter().find(|m| m.detect(root))
}

/// Chemin de `bin/javac` sous un dossier « maison » de JDK (`JAVA_HOME` ou
/// un candidat sdkman), s'il existe.
fn javac_path(home: &Path) -> Option<PathBuf> {
    let javac = home.join("bin").join("javac");
    javac.is_file().then_some(javac)
}

/// Liste les JDK gérés par sdkman sous `candidates_dir`
/// (`~/.sdkman/candidates/java`), triés par nom. L'entrée `current` (lien
/// vers un des autres candidats) est ignorée pour éviter un doublon.
fn sdkman_jdks(candidates_dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(read_dir) = fs::read_dir(candidates_dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read_dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n != "current")
        .collect();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let home = candidates_dir.join(&name);
            javac_path(&home).map(|javac| (format!("sdkman {name}"), javac))
        })
        .collect()
}

/// Cherche un exécutable `javac` dans les dossiers listés par `path_env`
/// (au format de la variable d'environnement `PATH`).
fn path_javac(path_env: &OsStr) -> Option<PathBuf> {
    env::split_paths(path_env).map(|dir| dir.join("javac")).find(|p| p.is_file())
}

fn push_unique(jdks: &mut Vec<Jdk>, seen: &mut HashSet<PathBuf>, label: String, javac: PathBuf) {
    let key = fs::canonicalize(&javac).unwrap_or_else(|_| javac.clone());
    if seen.insert(key) {
        jdks.push(Jdk { label, javac });
    }
}

/// Détecte les JDK disponibles sur le système, dans cet ordre de
/// priorité : candidats sdkman (`~/.sdkman/candidates/java`), `JAVA_HOME`,
/// puis `javac` trouvable dans le `PATH`. Un même JDK détecté par
/// plusieurs sources n'apparaît qu'une fois (déduplication par chemin
/// canonique).
pub fn detect_jdks() -> Vec<Jdk> {
    let mut jdks = Vec::new();
    let mut seen = HashSet::new();

    if let Some(home_dir) = env::var_os("HOME") {
        let candidates = PathBuf::from(home_dir).join(".sdkman/candidates/java");
        for (label, javac) in sdkman_jdks(&candidates) {
            push_unique(&mut jdks, &mut seen, label, javac);
        }
    }
    if let Some(java_home) = env::var_os("JAVA_HOME")
        && let Some(javac) = javac_path(Path::new(&java_home))
    {
        push_unique(&mut jdks, &mut seen, "JAVA_HOME".to_string(), javac);
    }
    if let Some(path_env) = env::var_os("PATH")
        && let Some(javac) = path_javac(&path_env)
    {
        push_unique(&mut jdks, &mut seen, "PATH".to_string(), javac);
    }

    jdks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "").unwrap();
    }

    #[test]
    fn find_java_files_scans_recursively() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("Main.java"));
        touch(&dir.path().join("src/pkg/Util.java"));
        touch(&dir.path().join("README.md"));

        let mut files: Vec<String> = find_java_files(dir.path())
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        files.sort();
        assert_eq!(files, vec!["Main.java".to_string(), "Util.java".to_string()]);
    }

    #[test]
    fn find_java_files_empty_when_no_java_file() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("notes.txt"));
        assert!(find_java_files(dir.path()).is_empty());
    }

    #[test]
    fn java_module_detects_only_when_java_files_present() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!JavaModule.detect(dir.path()));
        touch(&dir.path().join("Main.java"));
        assert!(JavaModule.detect(dir.path()));
    }

    #[test]
    fn javac_path_requires_bin_javac_to_exist() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(javac_path(dir.path()), None);
        touch(&dir.path().join("bin/javac"));
        assert_eq!(javac_path(dir.path()), Some(dir.path().join("bin/javac")));
    }

    #[test]
    fn sdkman_jdks_lists_candidates_and_skips_current_alias() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("17.0-tem/bin/javac"));
        touch(&dir.path().join("21.0-tem/bin/javac"));
        // Candidat sans javac : ne doit pas apparaître.
        fs::create_dir_all(dir.path().join("broken")).unwrap();
        // Alias `current` : ignoré même s'il contient un javac valide.
        touch(&dir.path().join("current/bin/javac"));

        let found: Vec<String> = sdkman_jdks(dir.path()).into_iter().map(|(label, _)| label).collect();
        assert_eq!(found, vec!["sdkman 17.0-tem".to_string(), "sdkman 21.0-tem".to_string()]);
    }

    #[test]
    fn path_javac_finds_first_match_in_path_dirs() {
        let dir_without = tempfile::tempdir().unwrap();
        let dir_with = tempfile::tempdir().unwrap();
        touch(&dir_with.path().join("javac"));

        let path_env = env::join_paths([dir_without.path(), dir_with.path()]).unwrap();
        assert_eq!(path_javac(&path_env), Some(dir_with.path().join("javac")));
    }

    #[test]
    fn path_javac_none_when_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path_env = env::join_paths([dir.path()]).unwrap();
        assert_eq!(path_javac(&path_env), None);
    }
}
