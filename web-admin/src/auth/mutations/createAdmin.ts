import sensei from "../../utils/sensei";

const createAdmin = async (username: string, alias: string, passphrase: string, start: boolean) => {
    return sensei.init({username, alias, passphrase, start})
}

export default createAdmin