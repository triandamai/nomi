export default async function run(args) { return 'Hello ' + args.name; }
(async () => { console.log(await run({name: 'World'})); })();