import sensei from "../../utils/sensei";

const createAdmin = async (username: string, alias: string, passphrase: string, electrumUrl: string, start: boolean) => {
    return sensei.init({username, alias, passphrase, electrumUrl, start})
}

export default createAdmin