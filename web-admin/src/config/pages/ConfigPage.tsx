import ConfigForm from "../components/ConfigForm";

const ConfigPage = () => {
  return (
    <div className="py-6">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-2xl font-semibold text-light-plum">Configuration</h1>
      </div>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
        <div className="py-4">
          <div className="bg-plum-100 shadow p-4 rounded-lg">
            <ConfigForm />
          </div>
        </div>
      </div>
    </div>
  );
};

export default ConfigPage;
