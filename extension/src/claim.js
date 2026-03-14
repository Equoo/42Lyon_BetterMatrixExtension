
async function getTimes() {
  const response = await fetch("http://127.0.0.1:15000/users/dderny/all");
  return await response.json();
}

const MAX_HOURS = 3.50;

function apply_needed_time(timePerHost) {
	for (const { host, total, total_ms } of timePerHost) {
		const card = document.querySelector("#" + host + "");
		if (!card) continue;

		const hours = total_ms / 3600000;
		const ratio = Math.floor(Math.min(hours / MAX_HOURS, 1) * 6) / 6;
		let hue = (ratio * 100);
		const lightness = (ratio * 35);

		if (ratio == 1)
		 	hue = 120;

		card.style.backgroundColor = `hsl(${hue}, 75%, ${lightness}%)`;
	}
}

function apply_drawer_info(timePerHost, timeLongest) {
	const drawer = document.querySelector("div[data-slot=drawer-content]");

	if (!drawer)
		return;

	const title = document.querySelector("div[data-slot=drawer-title]");
	let host = "";
	if (title)
		host = title.innerHTML.split(" ")[0];

	let info = document.querySelector("#drawer-time-info");
	if (!info) {
		info = document.createElement('div');
		info.classList = "text-card-foreground gap-6 rounded-xl border bg-muted/50 border-border/50 shadow-md flex flex-col items-center justify-center py-4";
		info.id = "drawer-time-info";

		content = document.createElement('div');
		content.classList = "flex gap-2 w-full px-4"; //justify-between
		info.append(content);

		const title = document.createElement('h4');
		title.classList = "text-lg font-bold";
		title.textContent = "Your logtime"
		content.append(title);

		data = document.createElement('div');
		data.classList = "flex flex-col gap-2 w-full px-4";
		content.append(data);

		const total = document.createElement('div');
		total.classList = "text-primary inline-block w-fit text-xs tabular-nums py-1";
		let totalHost = timePerHost.find(v => v.host == host);
		if (!totalHost) totalHost = {total: "Never used"};
		total.textContent = "Total: " + totalHost.total;
		data.append(total);

		const longest = document.createElement('div');
		longest.classList = "text-primary inline-block w-fit text-xs tabular-nums py-1";
		let longestTime = timeLongest.find(v => v.host == host);
		if (!longestTime) longestTime = {total: "Never used"};
		longest.textContent = "Longest session: " + longestTime.total
		data.append(longest);

		const header = document.querySelector("div[data-slot=drawer-header]");
		header.append(info);
	}

}

function claimable_count(timeLongest) {
	let count = 0;
	for (const { total_ms } of timeLongest) {
		const hours = total_ms / 3600000;
		if (hours > MAX_HOURS)
			count++;
	}

	let claimable = document.querySelector("#claimable");
	if (!claimable) {
		const header_r = document.querySelector("main div .ml-auto");
		claimable = document.createElement('div');
		claimable.classList = "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive focus-visible:border-ring focus-visible:ring-ring/50 relative inline-flex shrink-0 items-center justify-center gap-2 overflow-hidden rounded-md text-sm font-medium whitespace-nowrap outline-hidden transition-all select-none focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-background hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 dark:hover:bg-input/50 border shadow-2xs data-[active=true]:bg-accent data-[active=true]:text-accent-foreground dark:data-[active=true]:bg-accent h-9 px-4 py-2 has-[>svg]:px-3";
		claimable.id = "claimable";
		header_r.prepend(claimable);
	}
	claimable.innerHTML = count + "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"currentColor\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide-icon lucide lucide-star text-green-400\"><!----><path d=\"M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z\"></path><!----><!----><!----></svg>";
}

async function main() {
	const times = await getTimes();
	const timePerHost = times[0];
	const timeLongest = times[1];

	const element = document.querySelector('main');
	
	apply_needed_time(timeLongest);
	apply_drawer_info(timePerHost, timeLongest);
	claimable_count(timeLongest);
	
	const observer = new MutationObserver(() => {
		console.log('Changes stopped, executing function');
		
		observer.disconnect();
		
		apply_needed_time(timeLongest);
		apply_drawer_info(timePerHost, timeLongest);
		claimable_count(timeLongest);
		
		observer.observe(element, {
			childList: true,
			subtree: true,
			attributes: true
		});
	});
	
	observer.observe(element, {
		childList: true,
		subtree: true,
		attributes: true
	});
}

main().catch(console.error);
