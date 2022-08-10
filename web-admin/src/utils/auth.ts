const authProvider = {
  setup: false,
  macaroon: null,
  token: null,
  alias: null,
  init(passphrase: string, callback: (any) => void) {
    authProvider.setup = true;
    authProvider.macaroon = "secure";
    setTimeout(() => {
      callback(authProvider.getStatus());
    }, 100);
  },
  start(passphrase: string, callback: (any) => void) {
    setTimeout(() => {
      callback(authProvider.getStatus());
    }, 100); // fake async
  },
  stop(callback: (any) => void) {
    setTimeout(() => {
      callback(authProvider.getStatus());
    }, 100);
  },
  getStatus() {
    return {
      setup: authProvider.setup,
      macaroon: authProvider.macaroon,
      alias: authProvider.alias,
    };
  },
};

export { authProvider };
