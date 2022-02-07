import React from "react";
import { useNavigate, useLocation } from "react-router";
import { useAuth } from "../../contexts/auth";
import logo from "../../images/Icon-Lightning@2x.png";


const LoginPage = () => {
  let [submitting, setSubmitting] = React.useState<boolean>(false);
  let [submitError, setSubmitError] = React.useState<string>(null!);
  let navigate = useNavigate();
  let location = useLocation();
  let auth = useAuth();

  let from = location.state?.from?.pathname || "/admin/chain";

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setSubmitError(null);

    let formData = new FormData(event.currentTarget);
    let username = formData.get("username") as string;
    let passphrase = formData.get("passphrase") as string;

    try {
      await auth.login(username, passphrase);
      // Send them back to the page they tried to visit when they were
      // redirected to the login page. Use { replace: true } so we don't create
      // another entry in the history stack for the login page.  This means that
      // when they get to the protected page and click the back button, they
      // won't end up back on the login page, which is also really nice for the
      // user experience.
      navigate(from, { replace: true });
    } catch (e) {
      setSubmitError("invalid passphrase");
      setSubmitting(false);
    }
  }

  return (
    <div className="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="bg-plum-100 text-light-plum py-8 px-4 shadow sm:rounded-lg sm:px-10">
          <div className="sm:mx-auto sm:w-full sm:max-w-md">
            
              <img src={logo} alt="sensei logo" className="h-20 mx-auto" />
            

            <h2 className="mt-6 mb-6 text-center text-2xl font-bold text-gray-100">
              Login to your node
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
                className={`block text-sm font-medium ${
                  submitError ? "text-red-400" : "text-light-plum"
                }`}
              >
                Username
              </label>
              <div className="mt-1">
                <input
                  id="username"
                  name="username"
                  type="text"
                  required
                  className={`${
                    submitError ? "border-red-400" : "border-plum-200"
                  } bg-plum text-light-plum appearance-none block w-full px-3 py-2 border  rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm`}
                />
              </div>
            </div>
            <div>
              <label
                htmlFor="passphrase"
                className={`block text-sm font-medium ${
                  submitError ? "text-red-400" : "text-light-plum"
                }`}
              >
                Passphrase
              </label>
              <div className="mt-1">
                <input
                  id="passphrase"
                  name="passphrase"
                  type="password"
                  required
                  className={`${
                    submitError ? "border-red-400" : "border-plum-200"
                  } bg-plum text-light-plum appearance-none block w-full px-3 py-2 border  rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm`}
                />
              </div>
            </div>

            <div>
              <button
                type="submit"
                disabled={submitting}
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-orange hover:bg-orange-hover focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                {submitting && <span>...</span>}
                {!submitting && <span>Login</span>}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};

export default LoginPage;
