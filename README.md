# solana-validator-ip-tracker
track solana validator ip details for an epoch

## How to run
- you will need the core api key from [dp-ip](https://db-ip.com/api/core/)
- add the api key and rpc url as env variables or to a .env file (refer to .env.sample in the project)

```
RPC_URL=http://beta-solana.mainnetrpc.com
API_KEY=<DB_IP_API_KEY>
```
just run with cargo and a csv will be generated.

```
cargo run -r
```

## Output

| pubkey                                       | stake          | totalSlots | ipAddress     | continentCode | continentName | countryCode | countryName     | isEuMember | currencyCode | currencyName | phonePrefix | stateProvCode | stateProv     | district                   | city              | geonameId | gmtOffset | timeZone         | latitude | longitude | weatherCode | asNumber | asName     | isp                       | usageType | organization             |
|----------------------------------------------|----------------|------------|---------------|---------------|---------------|-------------|-----------------|------------|--------------|--------------|-------------|---------------|---------------|----------------------------|-------------------|-----------|-----------|------------------|----------|-----------|-------------|----------|------------|---------------------------|-----------|--------------------------|
| G4GT8z4AKWNoy3x6nuzxW83UfFXLXzrwn7DZQt4GvWdU | 31848352985660 | 40         | 64.130.50.45  | EU            | Europe        | DE          | Germany         | true       | EUR          | Euro         | 49          | HE            | Hesse         | Regierungsbezirk Darmstadt | Frankfurt am Main | 2925533   | 1         | Europe/Berlin    | 50.1995  | 8.68182   | GMXX0040    | 20326    | TERASWITCH | TeraSwitch Networks Inc.  | hosting   | TeraSwitch Networks Inc  |
| 66RAWQ8kUE95WpuJ7vkAS55TWfDo7ZonyTsLeGfs7dt3 | 74012625827801 | 84         | 217.69.13.37  | EU            | Europe        | FR          | France          | true       | EUR          | Euro         | 33          | IDF           | Île-de-France | Seine-Saint-Denis          | Aubervilliers     | 3036386   | 1         | Europe/Paris     | 48.9107  | 2.3884    | FRXX0007    | 20473    | AS-VULTR   | The Constant Company, LLC | hosting   | Vultr Holdings LLC Paris |
| E9hD3ikumJx1GVswDjnpCt6Uu4WG5mz1PDWCqdE5uhmo | 36469416419670 | 52         | 202.8.8.42    | EU            | Europe        | NL          | The Netherlands | true       | EUR          | Euro         | 31          | NH            | North Holland | Gemeente Amsterdam         | Amsterdam         | 2759794   | 1         | Europe/Amsterdam | 52.3734  | 4.89406   | NLXX0002    | 20326    | TERASWITCH | TeraSwitch Networks Inc.  | corporate | TeraSwitch Networks Inc  |
| 2icWF7TvxyycF7d1NHpMZYuJJqiRy2h7wmjFSbqUij1B | 53275558842233 | 76         | 64.130.52.202 | EU            | Europe        | NL          | The Netherlands | true       | EUR          | Euro         | 31          | NH            | North Holland | Gemeente Amsterdam         | Amsterdam         | 2759794   | 1         | Europe/Amsterdam | 52.3734  | 4.89406   | NLXX0002    | 20326    | TERASWITCH | TeraSwitch Networks Inc.  | hosting   | TeraSwitch Networks Inc  |
