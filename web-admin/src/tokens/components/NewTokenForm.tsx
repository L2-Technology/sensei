import { useNavigate } from "react-router";
import { Form, Input, Select } from "../../components/form";
import { z } from "zod";
import createAccessToken from "../mutations/createAccessToken";
import addMinutes from "date-fns/addMinutes";

export const CreateTokenInput = z.object({
  name: z.string(),
  scope: z.string(),
  expiresAt: z.string(),
  singleUse: z.string(),
});

const NewTokenForm = () => {
  let navigate = useNavigate();

  let singleUseOptions = [
    { value: "false", text: "Unlimited Use" },
    { value: "true", text: "Single Use" },
  ];

  let scopeOptions = [
    { value: "*", text: "All Scopes" },
    { value: "nodes/create", text: "Create Nodes" },
    {
      value: "nodes/create,nodes/list,nodes/delete,nodes/stop,nodes/start",
      text: "Node Management",
    },
    { value: "routing", text: "Routing" },
  ];

  let expirationOptions = [
    { value: `0`, text: "Never" },
    { value: `5`, text: "5 minutes" },
    { value: `60`, text: "1 hour" },
    { value: `360`, text: "6 hours" },
    { value: `${24 * 60}`, text: "24 hours" },
    { value: `${7 * 24 * 60}`, text: "1 week" },
    { value: `${2 * 7 * 24 * 60}`, text: "2 weeks" },
    { value: `${30 * 24 * 60}`, text: "30 days" },
    { value: `${60 * 24 * 60}`, text: "60 days" },
    { value: `${90 * 24 * 60}`, text: "90 days" },
    { value: `${180 * 24 * 60}`, text: "180 days" },
    { value: `${365 * 24 * 60}`, text: "1 year" },
    { value: `${2 * 365 * 24 * 60}`, text: "2 years" },
    { value: `${3 * 365 * 24 * 60}`, text: "3 years" },
  ];

  return (
    <Form
      submitText="Create Token"
      schema={CreateTokenInput}
      initialValues={{
        name: "",
        singleUse: "false",
        expiresAt: "0",
        scope: "*",
      }}
      noticePosition="top"
      layout="default"
      onSubmit={async ({ name, scope, expiresAt, singleUse }) => {
        let expiresAtInt = parseInt(expiresAt, 10);
        const expiresAtActual =
          expiresAtInt === 0
            ? 0
            : addMinutes(new Date(), expiresAtInt).getTime();

        try {
          await createAccessToken(
            name,
            scope,
            expiresAtActual,
            singleUse === "true"
          );
          navigate("/admin/tokens");
        } catch (e) {
          // TODO: handle error
        }
      }}
    >
      <Input autoFocus label="Name" name="name" />
      <Select label="Scope" name="scope" options={scopeOptions} />
      <Select label="Expiration" name="expiresAt" options={expirationOptions} />
      <Select label="Usage Limit" name="singleUse" options={singleUseOptions} />
    </Form>
  );
};

export default NewTokenForm;
