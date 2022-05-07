import { useNavigate } from "react-router";
import createNode from "../mutations/createNode";
import { Form, Input } from "../../components/form";
import { z } from "zod";

export const CreateNodeInput = z.object({
  username: z.string(),
  alias: z.string(),
  passphrase: z.string(),
});

const NewNodeForm = () => {
  let navigate = useNavigate();

  return (
    <Form
      submitText="Create Node"
      schema={CreateNodeInput}
      noticePosition="top"
      layout="default"
      onSubmit={async ({ alias, passphrase, username }) => {
        try {
          await createNode(username, alias, passphrase, true);
          navigate("/admin/nodes");
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <Input autoFocus label="Username" name="username" />
      <Input label="Alias" name="alias" />
      <Input label="Passphrase" name="passphrase" type="password" />
    </Form>
  );
};

export default NewNodeForm;
