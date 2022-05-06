import adminStartNode from "../mutations/adminStartNode";
import { Form, FORM_ERROR, Input } from "../../components/form";
import { z } from "zod";
import { PlayIcon } from "@heroicons/react/outline";
export const StartNodeInput = z.object({
  pubkey: z.string(),
  passphrase: z.string(),
});

const StartNodeForm = ({ pubkey, callback }) => {
  return (
    <>
      <div className="flex mb-4">
        <div className="mr-4">
          <div className="bg-green-100 p-2 rounded-full">
            <PlayIcon className="h-10 text-green-700 font-thin" />
          </div>
        </div>
        <div className="">
          <div className="font-semibold text-lg">Start Node</div>
          <div className="">
            Enter the passphrase to start and unlock this node
          </div>
        </div>
      </div>
      <Form
        submitText="Start Node"
        schema={StartNodeInput}
        noticePosition="bottom"
        layout="default"
        initialValues={{ pubkey, passphrase: "" }}
        onSubmit={async ({ pubkey, passphrase }) => {
          try {
            await adminStartNode(pubkey, passphrase);
            callback();
          } catch (e) {
            return { [FORM_ERROR]: "Invalid passphrase" };
          }
        }}
      >
        <Input type="hidden" name="pubkey" value={pubkey} />
        <Input label="Passphrase" name="passphrase" type="password" />
      </Form>
    </>
  );
};

export default StartNodeForm;
