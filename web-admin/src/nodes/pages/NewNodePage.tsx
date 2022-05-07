import NewNodeForm from "../components/NewNodeForm";
import { Link } from "react-router-dom";

const NewNodePage = () => {
  return (
    <div className="py-12">
      <div className="">
        <div className="pb-5 border-b border-gray-400 sm:flex sm:items-center sm:justify-between">
          <h3 className="text-2xl leading-6 font-medium text-light-plum">
            Create Node
          </h3>

          <div className="mt-3 sm:mt-0 sm:ml-4">
            <Link to="/admin/nodes" className="btn-ghost">
              Cancel
            </Link>
          </div>
        </div>
      </div>
      <div className="">
        <div className="py-4">
          <div className="bg-plum-100 text-light-plum shadow p-4 rounded-xl">
            <NewNodeForm />
          </div>
        </div>
      </div>
    </div>
  );
};

export default NewNodePage;
