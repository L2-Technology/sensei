import React from "react";
import { useNavigate, useLocation } from "react-router";
import { AlertMsg } from "src/components/ErrorAlert";
import Spinner from "src/components/Spinner";
import { useAuth } from "../../contexts/auth";
import logo from "../../images/Icon-Lightning@2x.png";

const AdminLoginPage = () => {
  let [submitting, setSubmitting] = React.useState<boolean>(false);
  let [submitError, setSubmitError] = React.useState<string>(null!);
  let navigate = useNavigate();
  let location = useLocation();
  let auth = useAuth();

  let from = location.state?.from?.pathname || "/admin/nodes";

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setSubmitError(null);

    let formData = new FormData(event.currentTarget);
    let username = formData.get("username") as string;
    let passphrase = formData.get("passphrase") as string;

    try {
      await auth.loginAdmin(username, passphrase);
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
        <div className="bg-plum-100 text-light-plum py-8 px-4 shadow sm:rounded-xl sm:px-10">
          <div className="sm:mx-auto sm:w-full sm:max-w-md">
            <img src={logo} alt="sensei logo" className="h-20 mx-auto" />

            <h2 className="mt-6 mb-6 text-center text-2xl font-bold text-gray-100">
              Login to the admin
            </h2>
          </div>

          {submitError && (
            <AlertMsg type="error" className="-mt-2 mb-2">
              {submitError}
            </AlertMsg>
          )}

          <form onSubmit={handleSubmit} action="#" method="POST">
            <fieldset disabled={submitting} className="space-y-6">
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
                    autoFocus
                    id="username"
                    name="username"
                    type="text"
                    required
                    className={`${submitError ? "!border-red-400" : ""} input`}
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
                    className={`${submitError ? "!border-red-400" : ""} input`}
                  />
                </div>
              </div>

              <div>
                <button
                  type="submit"
                  className="btn-orange w-full justify-center"
                >
                  {submitting ? <Spinner /> : "Login"}
                </button>
              </div>
            </fieldset>
          </form>
        </div>
      </div>
    </div>
  );
};

export default AdminLoginPage;
