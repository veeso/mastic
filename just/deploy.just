
[group('deploy')]
dfx_start: dfx_prepare_env dfx_stop
  # start the local dfx replica
  dfx start --background --clean 2> ./target/dfx.log

  # fabricate cycles
  wallet_principal=$(dfx identity get-wallet) && dfx ledger fabricate-cycles --t 1000000 --canister $wallet_principal
  sleep 2

[group('deploy')]
dfx_stop:
  dfx stop

[group('deploy')]
dfx_deploy_local install_mode="install":
  #!/usr/bin/env bash
  set -ex

  if [ "{{install_mode}}" = "install" ]; then
      just dfx_start
      # Install NNS
      dfx nns -q install
      echo "Creating canisters on local network"
      just create_canisters local 1000000000000000
      # install the orbit station canister
      echo "Deploying Orbit Station canister on local network"
      just deploy_orbit_local
      # Install the orchestrator canister
      echo "Deploying Orchestrator canister on local network"
      just deploy_canisters local "install"
  elif [ "{{install_mode}}" = "upgrade" ] || [ "{{install_mode}}" = "reinstall" ]; then
      just deploy_canisters local "{{install_mode}}"
  else
      echo "Usage: $0 <create|upgrade|reinstall>"
      exit 1
  fi

  just dfx_print_stats local

[group('deploy')]
dfx_log canister network="local":
  dfx canister logs --network={{network}} {{canister}}

[private]
dfx_print_stats network="local":
  #!/usr/bin/env bash
  set -e

  dfx_webserver_port=$(dfx info webserver-port)
  orbit_station_id=$(dfx canister --network={{network}} id orbit-station)
  directory_id=$(dfx canister --network={{network}} id directory)
  federation_id=$(dfx canister --network={{network}} id federation)
  frontend_id=$(dfx canister --network={{network}} id frontend)

  echo "webserver-port: $dfx_webserver_port"
  echo "orbit-station: $orbit_station_id"
  echo "directory: $directory_id"
  echo "federation: $federation_id"
  echo "frontend url: http://$frontend_id.raw.localhost:$dfx_webserver_port"

[private]
dfx_prepare_env: 
  dfx identity new --storage-mode=plaintext --force admin
  dfx identity use admin

[private]
create_canisters network cycles:
  dfx canister --network={{network}} create --with-cycles={{cycles}} orchestrator
  dfx canister --network={{network}} create --with-cycles={{cycles}} marketing
  dfx canister --network={{network}} create --with-cycles={{cycles}} frontend

[private]
deploy_canisters network install_mode:
  #!/usr/bin/env bash
  set -ex

  just build_all_canisters
