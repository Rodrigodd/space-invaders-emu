const sounds = [
    // new sound("sound/0.wav"),
    new sound("sound/1.wav"),
    new sound("sound/2.wav"),
    new sound("sound/3.wav"),
    new sound("sound/4.wav"),
    new sound("sound/5.wav"),
    new sound("sound/6.wav"),
    new sound("sound/7.wav"),
    new sound("sound/8.wav"),
];
const ufo = new sound("sound/0.wav", true);


export function play_sound(index) {
    sounds[index - 1].play();
}
export function start_ufo() {
    ufo.play();
}
export function stop_ufo() {
    ufo.stop();
}

function sound(src, loop) {
    this.sound = document.createElement("audio");
    this.sound.src = src;
    this.sound.setAttribute("preload", "auto");
    this.sound.setAttribute("controls", "none");
    if (loop) {
        this.sound.setAttribute("loop", "true")
    }
    this.sound.style.display = "none";
    document.body.appendChild(this.sound);
    this.play = function(){
        this.sound.play();
    }
    this.stop = function(){
        this.sound.pause();
    }
}