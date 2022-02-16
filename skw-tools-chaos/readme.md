## The Chaos Party

This is a simple Chaos Party game to spawn the SkyeKiwi Network. Only runable on a chain with `--dev` enabled. 

It will spawn 10-ish new accounts (i.e. "//1", "//2" ...) and fund them with some tokens from "//Alice". Then spawn a bunch of PM2 managed processes that will each submit 10 random contract calls to destination blockchain network. 

Run `yarn party` to start the party! Make tweaks to `main.ts` to better suite the specific needs of your testing process. 