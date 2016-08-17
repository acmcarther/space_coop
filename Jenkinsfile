node('master') {
    //docker.image("schickling/rust").inside {
        try {
            stage 'Checkout'
                checkout scm
            stage 'Version Check'
                sh 'rustc --version'
                sh 'cargo --version'
            stage 'Build'
                sh '(cd prototype2 && nix-shell --run "cargo build")'
            stage 'Test Root'
                sh '(cd prototype2 && nix-shell --run "cargo test")'
            stage 'Test Common'
                sh '(cd prototype2 && nix-shell --run "cd common; cargo test")'
            stage 'Test Client'
                sh '(cd prototype2 && nix-shell --run "cd client; cargo test")'
            stage 'Test Server'
                sh '(cd prototype2 && nix-shell --run "cd server; cargo test")'
            currentBuild.result = "SUCCESS"
        } catch (err) {
            currentBuild.result = "FAILURE"
            throw err
        }
    //}
}
