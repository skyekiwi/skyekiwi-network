FROM skyekiwi/skyekiwi-network:0.1

ARG EnvironmentVariable

WORKDIR /root/skyekiwi-network

EXPOSE 30333 9933 9944 443

# CMD [ "yarn", "railway:run", "validator" ]
CMD [ "./target/release/skyekiwi-node", "--tmp", "--dev", "--unsafe-ws-external", "--unsafe-rpc-external", "--prometheus-external", "--ws-port", "443" ]
