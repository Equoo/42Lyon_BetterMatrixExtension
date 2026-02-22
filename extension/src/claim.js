
async function getTimePerHost() {
  const response = await fetch("http://127.0.0.1:15000/users/dderny/longest");
  return await response.json();
}

function edit_page(timePerHost) {
	const MAX_HOURS = 4;

  for (const { host, total } of timePerHost) {
    const card = document.querySelector("#" + host + "");
    if (!card) continue; // skip if host element doesn't exist on page
	
	 // Extract hours
    const hoursMatch = total.match(/^(\d+)h/);
    const hours = hoursMatch ? parseInt(hoursMatch[1], 10) : 0;

    // Normalize 0 → 1
    const ratio = Math.min(hours / MAX_HOURS, 1);

    // Green (low) → Red (high)
    const hue = (ratio * 120); 
    // 120 = green, 0 = red

    const lightness = (ratio * 40); 
    // darker when higher

    card.style.backgroundColor = `hsl(${hue}, 75%, ${lightness}%)`;

    const us = document.querySelector("#" + host + " div div div div p");
	  if (us)
		  return;

    const content = document.querySelector("#" + host + " div div div div");
    content.innerHTML += "<p class=\"text-center text-xs font-medium text-primary h-6 flex items-center whitespace-nowrap svelte-9k688j\">" + total + "</p>";
  }
}

async function main() {
	const timePerHost = await getTimePerHost();
	edit_page(timePerHost);
	
	const element = document.getElementById('clusters-container');
	
	const observer = new MutationObserver(() => {
		console.log('Changes stopped, executing function');
		
		// Disconnect observer before making changes
		observer.disconnect();
		
		// Execute your functions
		edit_page(timePerHost);
		
		// Reconnect observer after changes are done
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
