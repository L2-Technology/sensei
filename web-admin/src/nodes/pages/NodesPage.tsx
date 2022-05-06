import NodesListCard from "../components/NodesList";
import { Link } from "react-router-dom";

const NodesPage = () => {
  return (
    <div className="py-5">
      <div className="">
        <div className="pb-5 border-b border-plum-200 sm:flex sm:items-center sm:justify-between">
          <h3 className="text-2xl leading-6 font-medium text-light-plum">
            Nodes
          </h3>
          <div className="mt-3 sm:mt-0 sm:ml-4">
            <Link to="/admin/nodes/new" className="btn-orange">
              Create new node
            </Link>
          </div>
        </div>
      </div>
      <div className="py-4 relative">
        <div className="bg-gray-accent2 -mx-4 sm:mx-0 sm:rounded-xl overflow-x-auto">
          <NodesListCard />
        </div>
      </div>
    </div>
  );
};

export default NodesPage;
