import React from "react";
import { useNavigate } from "react-router";
import { useAuth } from "../../contexts/auth";
import logo from "../../images/Icon-Lightning@2x.png";
import Spinner from "src/components/Spinner";


const UsernamePassphraseStep = () => {
  let [submitting, setSubmitting] = React.useState<boolean>(false);
  let navigate = useNavigate();
  let auth = useAuth();

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true)

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
    setSubmitting(false)
    await navigate("/admin/chain");
  }

  return (
    <div className="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="bg-plum-100 text-light-plum py-8 px-4 shadow sm:rounded-xl sm:px-10">
          <div className="sm:mx-auto sm:w-full sm:max-w-md">
              <img src={logo} alt="sensei logo" className="h-20 mx-auto" />
            
            <h2 className="mt-6 mb-6 text-center text-2xl font-bold text-gray-100">
              Setup your node
            </h2>
          </div>
          <form
            onSubmit={handleSubmit}
            action="#"
            method="POST"
          >
           <fieldset disabled={submitting} className="space-y-6">
            <div>
              <label
                htmlFor="username"
                className="block text-sm font-medium text-light-plum"
              >
                Username
              </label>
              <div className="mt-1">
                <input
                  autoFocus
                  id="username"
                  name="username"
                  type="text"
                  placeholder="admin"
                  required
                  className="input"
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
                  className="input"
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
                  className="input"
                />
              </div>
            </div>
            <div>
              <button
                type="submit"
                disabled={submitting}
                className="btn-orange w-full justify-center"
              >
                {submitting ? <Spinner/> : "Setup"}
              </button>
            </div>
            </fieldset>
          </form>
        </div>
      </div>
    </div>
  );
};

export default UsernamePassphraseStep;
