# The Compute Layer of the BRC
This repo contains proof of all the brainfuck we had to go through on the server side to pull off hosting an entirely automated version of the BRC

## Workflows
- Builds and pushes worker image to https://hub.docker.com/r/steakfisher1/brc-worker
- Builds and pushes controller image to https://hub.docker.com/r/steakfisher1/brc-controller
- Builds an Ubuntu binary of the rust daemon and updates the private and public BRC boilerplate repos

## Controller
Acts as sort of a reverse proxy, that takes in requests via the /commit and /upgrade endpoints and accordingly puts it into the Proposal and Divorce queues respectively.

## Daemon
Not actually a daemon, we just wanted to sound fancy. Basically just runs on both the client (the boilerplate) and server environments in order to generate testcases, respective expected outputs and then calibrate the environment in order to benchmark each and every indidivual output in [blazingly fast speeds.](https://www.youtube.com/theprimeagen)

## Deploy
The fucked up approach for deployment of the infrastructure we had to take because of [AZURE'S BULLSHIT](https://x.com/TheDevyashSaini/status/1901740907457851784). Deploys 7 servers split across 3 different Azure accounts so as to not hti their ridiculous 6vCPU regional quota limit.

## Docker
Hosting the event like normal people didn't cut it for us, so this contains base Docker image we're using to supply contestants with a custom no GIL Python build.

## Terraform
The ACTUAL approach that SHOULD have worked had it not been for Azure trying to do whatever it takes to mentally break us. The terraform deployment files consisting of a proper k8s cluster with 3 node pools 2 of them which are auto scaling using KEDA.

## Worker
The worker that wraps the daemon in the production server, which continously polls the RabbitMQ Queue for events. Upon receiving an event builds and runs the Docker image which would proceed to call the Daemon which would then take over helping replicate the local dev environment to the prod one.
