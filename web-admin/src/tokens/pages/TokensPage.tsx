import TokensListCard from "../components/TokensList";
import { Link } from "react-router-dom";

const TokensPage = () => {
  return (
    <div className="py-12">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="pb-5 border-b border-plum-200 sm:flex sm:items-center sm:justify-between">
          <h3 className="text-2xl leading-6 font-medium text-light-plum">
            Tokens
          </h3>
          <div className="mt-3 sm:mt-0 sm:ml-4">
            <Link
              to="/admin/tokens/new"
              className="btn-orange"
            >
              Create new token
            </Link>
          </div>
        </div>
      </div>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
        <div className="py-4">
          <TokensListCard />
        </div>
      </div>
    </div>
  );
};

export default TokensPage;
