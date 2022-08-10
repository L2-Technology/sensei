import SenseiClient from "@l2-technology/sensei-client";

const client = new SenseiClient({
  basePath: `${
    process.env.NODE_ENV === "development" ? "http://localhost:5401" : ""
  }`,
});

export default client;
