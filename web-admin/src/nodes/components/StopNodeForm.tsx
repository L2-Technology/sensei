import adminStopNode from "../mutations/adminStopNode";
import { Form, Input } from "../../components/form";
import { z } from "zod";

export const StopNodeInput = z.object({
  pubkey: z.string(),
});

const StopNodeForm = ({ pubkey }) => {
  return (
    <Form
      submitText="Stop Node"
      schema={StopNodeInput}
      noticePosition="top"
      layout="default"
      initialValues={{ pubkey }}
      onSubmit={async ({ pubkey }) => {
        try {
          await adminStopNode(pubkey);
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <Input type="hidden" name="pubkey" value={pubkey} />
    </Form>
  );
};

export default StopNodeForm;
