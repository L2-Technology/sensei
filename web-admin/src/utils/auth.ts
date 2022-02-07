const authProvider = {
  created: false,
  running: false,
  macaroon: null,
  alias: null,
  create(alias: string, passphrase: string, callback: (any) => void) {
    authProvider.created = true;
    authProvider.macaroon = "secure";
    authProvider.alias = alias;
    setTimeout(() => {
      callback(authProvider.getStatus())
    }, 100);
  },
  start(passphrase: string, callback: (any) => void) {
    authProvider.running = true;
    setTimeout(() => {
      callback(authProvider.getStatus())
    }, 100); // fake async
  },
  stop(callback: (any) => void) {
    authProvider.running = false;
    setTimeout(() => {
      callback(authProvider.getStatus())
    }, 100);
  },
  getStatus() {
    return {
      created: authProvider.created,
      running: authProvider.running,
      macaroon: authProvider.macaroon,
      alias: authProvider.alias
    }
  }
};

export { authProvider };
