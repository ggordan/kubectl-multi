use std::io::Write;
use std::process::Stdio;

fn is_namespace_arg(v: &String) -> bool {
    v == "-n" || v == "--namespace" || v.starts_with("--namespace")
}

fn main() {

    // if no namespaces provided, offload to kubectl straight away
    let namespaces = get_namespaces();
    if namespaces == "" {
        std::process::Command::new("kubectl")
            .arg("get")
            .args(std::env::args().collect::<Vec<_>>()[1..].to_vec())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .unwrap();
        return;
    }

    let ns: Vec<String> = namespaces
        .split(",")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let args: Vec<String> = std::env::args()
    .filter(|arg| !is_namespace_arg(&arg))
    .filter(|arg| arg != &namespaces)
    .collect();

    run(ns, args[1..].to_vec());
}

// proxies to key pods command
fn run(namespaces: Vec<String>, args: Vec<String>) {
    let mut children = vec![];
    for namespace in namespaces {
        let args = args.clone();
        children.push(std::thread::spawn(move|| {
            let mut out = std::process::Command::new("kubectl")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .arg("get")
                .arg("-n")
                .arg(namespace)
                .args(args)
                .output()
                .unwrap();
        }));
    }
    for child in children {
        child.join();
    }
}

// returns the namespaces provided (if any)
fn get_namespaces() -> String {
    std::env::args()
        .enumerate()
        .find(|(_, v)| is_namespace_arg(&v))
        .map(|(i, _)| std::env::args().collect::<Vec<String>>()[i as usize + 1].to_owned())
        .unwrap_or(String::new())
}