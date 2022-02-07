import UnusedAddress from "../components/UnusedAddress";

const FundPage = () => {
  return (
    <div className="py-6">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-2xl font-semibold text-plum-light text-center">Fund Node</h1>
      </div>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
        <div className="py-4">
          <UnusedAddress />
        </div>
      </div>
    </div>
  );
};

export default FundPage;
