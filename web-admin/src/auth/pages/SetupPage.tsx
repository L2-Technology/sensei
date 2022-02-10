import { useNavigate } from "react-router";
import { useAuth } from "../../contexts/auth";
import logo from "../../images/Icon-Lightning@2x.png";

const UsernamePassphraseStep = () => {
  let navigate = useNavigate();
  let auth = useAuth();

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    let formData = new FormData(event.currentTarget);
    let username = formData.get("username") as string;
    let passphrase = formData.get("passphrase") as string;
    let alias = formData.get("alias") as string;

    await auth.create(
      username,
      alias,
      passphrase,
      true
    );
    await navigate("/admin/chain");
  }

  return (
    <div className="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="bg-plum-100 text-light-plum py-8 px-4 shadow sm:rounded-lg sm:px-10">
          <div className="sm:mx-auto sm:w-full sm:max-w-md">
              <img src={logo} alt="sensei logo" className="h-20 mx-auto" />
            
            <h2 className="mt-6 mb-6 text-center text-2xl font-bold text-gray-100">
              Setup your node
            </h2>
          </div>
          <form
            onSubmit={handleSubmit}
            className="space-y-6"
            action="#"
            method="POST"
          >
            <div>
              <label
                htmlFor="username"
                className="block text-sm font-medium text-light-plum"
              >
                Username
              </label>
              <div className="mt-1">
                <input
                  id="username"
                  name="username"
                  type="text"
                  placeholder="admin"
                  required
                  className="bg-plum text-light-plum appearance-none block w-full px-3 py-2 border border-plum-200 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                />
              </div>
            </div>
            <div>
              <label
                htmlFor="alias"
                className="block text-sm font-medium text-light-plum"
              >
                Alias
              </label>
              <div className="mt-1">
                <input
                  id="alias"
                  name="alias"
                  type="text"
                  placeholder="Satoshi"
                  required
                  className="bg-plum text-light-plum appearance-none block w-full px-3 py-2 border border-plum-200 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                />
              </div>
            </div>
            <div>
              <label
                htmlFor="passphrase"
                className="block text-sm font-medium text-light-plum"
              >
                Passphrase
              </label>
              <div className="mt-1">
                <input
                  id="passphrase"
                  name="passphrase"
                  type="password"
                  required
                  className="bg-plum text-light-plum appearance-none block w-full px-3 py-2 border border-plum-200 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                />
              </div>
            </div>
            <div>
              <button
                type="submit"
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-orange hover:bg-orange-hover focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                Setup
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};

export default UsernamePassphraseStep;
