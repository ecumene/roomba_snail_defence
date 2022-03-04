let locations = [
  [55, 209],
  [90, 76],
  [181, 91],
  [156, 185],
  [134, 231],
  [312, 191],
  [245, 34],
  [389, 58],
  [390, 120],
  [441, 176],
  [254, 125],
  [543, 220],
  [638, 210],
  [588, 160],
  [744, 165],
  [698, 188],
  [438, 36],
  [596, 15],
  [783, 63],
  [793, 151],
  [858, 147],
  [827, 66],
  [892, 49],
  [842, 43],
  [925, 91],
  [958, 99],
];

let points = 1000;
let buildings = [];

// const send = document.getElementById("send");
// const chat = document.getElementById("chat");
const text = document.getElementById("text");
const uri = "ws://" + location.host + "/chat";

let ws = new WebSocket(uri);

function message(data) {
  const parsed = JSON.parse(data);
  if (parsed.type == "killed") {
    points += 10;
    console.log(points);
  }
  if (parsed.type == "turret") {
    buildings.push(parsed);
    locations = locations.filter(([x, y]) => !(x == parsed.x && y == parsed.y));
    flushLocations();
  }
}

ws.onopen = function () {
  // chat.innerHTML = "<p><em>Connected!</em></p>";
};

ws.onmessage = function (msg) {
  message(msg.data);
};

ws.onclose = function () {
  // chat.getElementsByTagName("em")[0].innerText = "Disconnected!";
  setTimeout(() => {
    // chat.innerHTML = "<p><em>Connecting...</em></p>";
    ws = new WebSocket(uri);
  }, 3000);
};

// send.onclick = function () {
//   const msg = text.value;
//   ws.send(msg);
//   text.value = "";
//   message("<You>: " + msg);
// };

const buttonTurret = document.getElementById("button-turret");
const buttonStun = document.getElementById("button-stun");
const modal = document.getElementById("build-modal");

let lastLocation = null;
modal.style.display = "none";

const onIndicatorClick = (location, [scaledX, scaledY]) => {
  modal.style.display = "block";
  lastLocation = location;
  modal.style.top = scaledY + "px";
  modal.style.left = scaledX + "px";
};

const makeTurret = () => {
  if (!lastLocation) throw new Error();
  return {
    x: lastLocation[0],
    y: lastLocation[1],
    type: "turret",
  };
};

const point_to_buy = 100;

buttonTurret.addEventListener("click", () => {
  if (points >= point_to_buy) {
    points -= point_to_buy;
    ws.send(JSON.stringify(makeTurret()));
    lastLocation = null;
    modal.style.display = "none";
  } else {
    alert("You don't have enough points");
  }
});

buttonStun.addEventListener("click", () => {
  ws.send("turret");
});

const map = document.getElementById("map");

const body = document.getElementsByTagName("body")[0];

const typeToImage = (image) => {
  switch (image) {
    case "turret":
      return "turret1.png";
    case "stun":
      return "stun.png";
    default:
      throw new Error();
  }
};

const flushLocations = () => {
  const previousLocations = document.getElementsByClassName("indicator");
  [...previousLocations].forEach((location) => location.remove());
  locations.forEach(([x, y]) => {
    const indicator = document.createElement("div");
    indicator.className = "indicator";
    const scaledX = x * (map.offsetWidth / map.naturalWidth);
    const scaledY =
      (map.naturalHeight - y) * (map.offsetHeight / map.naturalHeight);
    indicator.style.top = `${scaledY - 16}px`;
    indicator.style.left = `${scaledX - 16}px`;
    indicator.addEventListener("click", () =>
      onIndicatorClick([x, y], [scaledX, scaledY])
    );
    document.body.appendChild(indicator);
  });

  buildings.forEach(({ x, y, type }) => {
    const building = document.createElement("img");
    building.src = `/assets/${typeToImage(type)}`;
    building.className = "building";
    const scale = map.offsetWidth / map.naturalWidth;
    const scaledX = x * scale;
    const scaledY = (map.naturalHeight - y) * scale;
    const width = 32 * scale;
    building.style.top = `${scaledY - width / 2 - 5 * scale}px`;
    building.style.left = `${scaledX - width / 2}px`;
    building.style.width = `${width}px`;
    building.style.height = `${width}px`;
    document.body.appendChild(building);
  });
};

flushLocations();
