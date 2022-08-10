import OpenChannelForm from "../components/OpenChannelForm";
import { Link } from "react-router-dom";

const OpenChannelPage = () => {
  return (
    <div className="py-12">
      <div className="">
        <div className="pb-5 border-b border-gray-400 sm:flex sm:items-center sm:justify-between">
          <h3 className="text-2xl leading-6 font-medium text-light-plum">
            Open Channel
          </h3>

          <div className="mt-3 sm:mt-0 sm:ml-4">
            <Link to="/channels" className="btn-ghost">
              Cancel
            </Link>
          </div>
        </div>
      </div>
      <div className="">
        <div className="py-4">
          <div className="bg-plum-100 text-light-plum shadow p-4 rounded-xl">
            <OpenChannelForm />
          </div>
        </div>
      </div>
    </div>
  );
};

export default OpenChannelPage;
