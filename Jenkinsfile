node('master') {
    //docker.image("schickling/rust").inside {
        try {
            stage 'Checkout'
                checkout scm
            stage 'Version Check'
                sh 'rustc --version'
                sh 'cargo --version'
            stage 'Build'
                sh '(cd prototype2 && cargo build)'
            stage 'Test Root'
                sh '(cd prototype2 && cargo test)'
            stage 'Test Subcrates'
                sh '(cd prototype2 && cargo test -p common -p client -p server)'
            currentBuild.result = "SUCCESS"
        } catch (err) {
            currentBuild.result = "FAILURE"
            throw err
        }
    //}
}
