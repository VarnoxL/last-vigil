import * as THREE from 'three'

const renderer = new THREE.WebGLRenderer()
renderer.setPixelRatio(window.devicePixelRatio)
renderer.setSize(window.innerWidth, window.innerHeight)
renderer.setClearColor(0x0b0b10)
document.body.appendChild(renderer.domElement)

const scene = new THREE.Scene()
const camera = new THREE.OrthographicCamera()

function resize() {
  const w = window.innerWidth, h = window.innerHeight
  renderer.setSize(w, h)
  Object.assign(camera, { left: -w / 2, right: w / 2, top: h / 2, bottom: -h / 2 })
  camera.updateProjectionMatrix()
}
window.addEventListener('resize', resize)
resize()

renderer.setAnimationLoop(() => renderer.render(scene, camera))
