FROM skyekiwi/skyekiwi-network:latest

ARG EnvironmentVariable

WORKDIR /root/skyekiwi-network

EXPOSE 30333 9933 9944 443


CMD [ "yarn", "railway:run", "validator" ]
