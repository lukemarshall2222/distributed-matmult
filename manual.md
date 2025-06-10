To run the Vanilla Rust implementation of the distributed matrix multiplication:

Dependencies:
Docker,
Docker Compose,
Rust,
Cargo

If all dependencies are installed and your docker desktop is open, it should run relatively easily.

1. Open a terminal in the distributed-matmult directory
2. Run the command `docker compose build` It may take a few minutes to build.
3. After it is done building, runt the command `docker compose up` This will start up the images and they will act as a local network of running containers.
4. Open a new terminal, in the distributed-matmult again.
Run the command `curl_cmd.sh` to run a curl command to execute a matrix multiplication in the application. You mayneed to change the permissions of curl_cmd.sh to make it executable.
5. To change the matrices being multiplied, change them in the curl_cmd.sh script and rerun it.
6. To exit the docker compose application, go back to the original terminal and press `ctrl-c`.

To run the Hydro implmentation of the distributed matrix multiplication: 

Dependencies:
Rust,
Cargo,
Hydro (should come with the prior two)

1. Open a terminal in the hydro_matmult directory.
2. Run the command `cargo run --examples cluster-matmult` This may cause quite a few libraries to be downloaded, but will eventually print out the result matrix.
3. To change the matrices being multiplied, you must go into the src/cluster_matmult.rs, find the hard coded result matrix dimensions and matrices, and change them and the dimensions to the target matrrices and their result dimensions.
4. To exit after the Hydro application completes, press `ctrl-c` in the terminal. 

