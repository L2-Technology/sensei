import { Link } from "react-router-dom";
import PeersList from "../components/PeersList";

const PeersPage = () => {
  return (
    <div className="py-12">
      <div className="">
        <div className="pb-5 border-b border-plum-200 sm:flex sm:items-center sm:justify-between">
          <h3 className="text-2xl leading-6 font-medium text-light-plum">
            Peer Directory
          </h3>
          <div className="mt-3 sm:mt-0 sm:ml-4">
            <Link to="/peers/new" className="btn-orange">
              Add Peer
            </Link>
          </div>
        </div>
      </div>
      <div className="py-4 relative">
        <div className="bg-gray-accent2 -mx-4 sm:mx-0 sm:rounded-xl overflow-x-auto">
          <PeersList />
        </div>
      </div>
    </div>
  );
};

export default PeersPage;
