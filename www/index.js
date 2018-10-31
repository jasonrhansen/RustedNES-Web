import { System } from "rustednes-web";
import { memory } from "rustednes-web/rustednes_web_bg";
import * as THREE from 'three';

let system = null;
const textureWidth = System.frame_width();
const textureHeight = System.frame_height();
const infoDiv = document.getElementById("info");
const romListDiv = document.getElementById("romList");

const renderer = new THREE.WebGLRenderer();
const canvas = renderer.domElement;
document.body.appendChild(canvas);
const scene = new THREE.Scene();
const camera = new THREE.OrthographicCamera( textureWidth / - 2, textureWidth / 2, textureHeight / 2, textureHeight / - 2, 1, 1000 );
const geometry = new THREE.PlaneGeometry(textureWidth, textureHeight);
const material = new THREE.MeshBasicMaterial();
const plane = new THREE.Mesh(geometry, material);
plane.position.z = -5;
scene.add(plane);

const drawFrame = () => {
  const framePtr = system.get_frame();
  const data = new Uint8Array(memory.buffer, framePtr, textureWidth * textureHeight * 4);
  const texture = new THREE.DataTexture(data, textureWidth, textureHeight, THREE.RGBAFormat);
  texture.needsUpdate = true;
  texture.flipY = true;
  material.map = texture;
  renderer.render(scene, camera);
};

let prevTimestamp = null;

const resizeRenderer = () => {
  const textureRatio = textureWidth / textureHeight;
  const windowRatio = window.innerWidth / window.innerHeight;

  if (windowRatio > textureRatio) {
    renderer.setSize(window.innerHeight * textureHeight / textureWidth, window.innerHeight);
  } else {
    renderer.setSize(window.innerWidth, window.innerWidth * textureRatio);
  }
  camera.updateProjectionMatrix();
};

resizeRenderer();

window.addEventListener('resize', () => {
  resizeRenderer();
});

window.addEventListener("focus", event => {
  prevTimestamp = null;
});

window.addEventListener("blur", event => {
  prevTimestamp = null;
});

window.addEventListener("keydown", event => {
  system.key_down(event.keyCode);
});

window.addEventListener("keyup", event => {
  system.key_up(event.keyCode);
});

let animationId = null;

const isPaused = () => {
  return animationId === null;
};

const play = () => {
  animationId = requestAnimationFrame(renderLoop);
}

const pause = () => {
  if (animationId) {
    animationId.cancelAnimationFrame();
  }
  animationId = null;
}

const fetchRomList = () => {
  var req = new XMLHttpRequest();
  req.addEventListener( "load" , () => {
    let response = JSON.parse(req.responseText);
    for (let i = 0; i < response.length; i++) {
      let name = response[i].name;
      let file = response[i].file;
      let btn = document.createElement("BUTTON");        // Create a <button> element
      let t = document.createTextNode(name);       // Create a text node
      btn.appendChild(t);
      btn.addEventListener("click", () => {
        loadRom(file);
      });
      romListDiv.appendChild(btn);
    }
  });
  req.open( "GET", "roms/index.json" );
  req.send();
};

const loadRom = file => {
  var req = new XMLHttpRequest();
  req.addEventListener( "load" , function() {
    var romData = new Uint8Array(req.response);
    system = System.new(romData);
    prevTimestamp = null;
    play();
  });
  req.open( "GET", "roms/" + file);
  req.responseType = "arraybuffer";
  req.send();
}

const renderLoop = timestamp => {
  if (prevTimestamp != null) {
    const timeDelta = timestamp - prevTimestamp;
    if (timeDelta < 35) {
      if (system.run_for(Math.round(timeDelta * 1000))) {
        drawFrame();
      }
    }
  }

  prevTimestamp = timestamp;
  animationId = requestAnimationFrame(renderLoop);
};

fetchRomList();
