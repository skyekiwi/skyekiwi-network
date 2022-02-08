FROM skyekiwi/skyekiwi-network:latest

ARG EnvironmentVariable

WORKDIR /root/skyekiwi-network

EXPOSE 30333 9933 443


CMD [ "./target/release/skyekiwi-node", "--tmp", "--dev", "--unsafe-ws-external", "--unsafe-rpc-external", "--prometheus-external", "--ws-port", "443" ]

