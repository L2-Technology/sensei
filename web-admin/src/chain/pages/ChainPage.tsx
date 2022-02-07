import OnChainBalance from "../components/OnChainBalance";
import TransactionsList from "src/transactions/components/TransactionsList";
const ChainPage = () => {
  return (
    <div className="py-6">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-2xl font-semibold text-plum-light">Balance</h1>
      </div>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
        <div className="py-4">
          <div className="mb-8">
            <OnChainBalance />
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-2xl font-semibold text-plum-light">Transactions</h1>
      </div>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
        <div className="py-4">
          <div className="mb-8">
            <TransactionsList />
          </div>
        </div>
      </div>
    </div>
  );
};

export default ChainPage;
