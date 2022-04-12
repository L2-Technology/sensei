import CreateInvoiceForm from "../components/CreateInvoiceForm";
import PaymentsList from "../components/PaymentsList";
const ReceiveMoneyPage = () => {
  return (
    <div className="py-6">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-2xl font-semibold text-light-plum">Receive Money</h1>
      </div>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
        <div className="py-4">
          <div className="bg-plum-100 text-light-plum shadow p-4 rounded-xl">
            <CreateInvoiceForm />
          </div>
        </div>
      </div>

      <div className="mt-8 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-2xl font-semibold text-light-plum">
          Incoming Payments
        </h1>
      </div>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
        <div className="py-4">
          <PaymentsList origin="incoming" />
        </div>
      </div>
    </div>
  );
};

export default ReceiveMoneyPage;
