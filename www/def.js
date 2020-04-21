export function set_volume(v) {
    return (window.audio.volume = v);
}

export function dfhh() {
    window.audio2.currentTime = 0;
    window.audio2.play();
}

export function duue() {
    window.audio3.currentTime = 0;
    window.audio3.play();
}

export function quipp() {
    window.audio4.currentTime = 0;
    window.audio4.play();
}

let death_notified = false;

export function exeunt(time) {
    if (death_notified) return;
    death_notified = true;
    const exit = document.createElement("div");
    const breaker0 = document.createElement("div");
    breaker0.className = "break";
    exit.appendChild(breaker0);
    const breaker1 = document.createElement("div");
    breaker1.className = "break";
    exit.appendChild(breaker1);
    const text = document.createElement("div");
    text.id = "top_line";
    text.innerText = "You kept the fire alive for " + time + " seconds.";
    text.style.color = "rgba(0,0,0,0)";
    exit.appendChild(text);
    const breaker2 = document.createElement("div");
    breaker2.className = "break";
    exit.appendChild(breaker2);
    if (time > 100) {
        const bottom_text = document.createElement("div");
        bottom_text.id = "bottom_line";
        bottom_text.innerText = "Good job!";
        bottom_text.style.color = "rgba(0,0,0,0)";
        exit.appendChild(bottom_text);
        const breaker3 = document.createElement("div");
        breaker3.className = "break";
        exit.appendChild(breaker3);
    }
    document.body.appendChild(exit).focus();
    exit.className = "exeunt";
}
