import { SvgSenseiMark } from "../../components/icons";
import { useNavigate } from "react-router";
import { useAuth } from "../../contexts/auth";
import React, { useState } from "react";
import logo from "../../images/Icon-Lightning@2x.png";

const UsernamePassphraseStep = ({ data, setData }) => {
  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setData({
      ...data,
      step: data.step + 1,
    });
  }

  function handleOnChange(event) {
    setData({
      ...data,
      [event.target.name]: event.target.value,
    });
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
                  value={data.username}
                  onChange={handleOnChange}
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
                  value={data.alias}
                  onChange={handleOnChange}
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
                  value={data.passphrase}
                  onChange={handleOnChange}
                  className="bg-plum text-light-plum appearance-none block w-full px-3 py-2 border border-plum-200 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                />
              </div>
            </div>
            <div>
              <button
                type="submit"
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-orange hover:bg-orange-hover focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                Next
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};

const BackendStep = ({ data, setData }) => {
  let navigate = useNavigate();
  let auth = useAuth();

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    await auth.create(
      data.username,
      data.alias,
      data.passphrase,
      data.electrum_url,
      true
    );
    await navigate("/admin/chain");
  }

  function handleOnChange(event) {
    setData({
      ...data,
      [event.target.name]: event.target.value,
    });
  }

  return (
    <div className="min-h-full flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="bg-plum text-light-plum py-8 px-4 shadow sm:rounded-lg sm:px-10">
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
                htmlFor="electrum_url"
                className="block text-sm font-medium text-light-plum"
              >
                Electrum URL
              </label>
              <div className="mt-1">
                <input
                  id="electrum_url"
                  name="electrum_url"
                  type="text"
                  placeholder="localhost:60401"
                  required
                  value={data.electrum_url}
                  onChange={handleOnChange}
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

const SetupPage = () => {
  let [data, setData] = useState({
    step: 0,
    alias: "",
    username: "",
    passphrase: "",
    electrum_url: "",
  });

  const stepComponentMap = {
    0: UsernamePassphraseStep,
    1: BackendStep,
  };

  const Component = stepComponentMap[data.step];

  return <Component data={data} setData={setData} />;
};

export default SetupPage;
