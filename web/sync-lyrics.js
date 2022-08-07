Vue.createApp({
    data() {
        return {
            step: 1,
            videoUrl: '',
            lyrics: '',
            player: null,
            // objects like {text: string, times: number[], isSpace: boolean}
            lyricsStops: [],
            undoStack: [],
        }
    },
    methods: {
        checkVideoUrl(event) {
            if (!this.videoId) {
                event.target.setCustomValidity('Could not extract video id from URL')
            } else {
                event.target.setCustomValidity('')
            }
            event.target.reportValidity()
        },
        submitFormStep1() {
            this.step = 2

            const wordsAndSpaces = Array.from(this.lyrics.matchAll(/\s+|\S+/g))
            this.lyricsStops = wordsAndSpaces.map(([text]) => ({
                text,
                times: [],
                isSpace: text.match(/\s/) !== null
            }))

            Vue.nextTick(() => {
                YT.ready(() => {
                    this.player = new YT.Player('youtube-player', {
                        height: '405',
                        width: '720',
                        videoId: this.videoId,
                        playerVars: {
                            // Documented at https://developers.google.com/youtube/player_parameters
                            autoplay: '1',
                            fs: '0',
                        },
                        events: {
                            onReady: () => {
                                this.player.setPlaybackRate(0.75)
                            }
                        }
                    })
                })
            })
        },
        addStop(stop) {
            if (!stop.isSpace) {
                stop.times.push(Math.round(1e3 * this.player.getCurrentTime()))
                this.undoStack.push(stop)
            }
        },
        undoStop() {
            const stop = this.undoStack.pop()
            stop.times.pop()
        },
        finish() {
            this.step = 3
        }
    },
    computed: {
        lyricsStopWords() {
            const stops =  this.lyricsStops.map(each => {
                if (each.isSpace) {
                    return each.text
                } else {
                    const word = {word: each.text}
                    if (each.times.length > 0) {
                        word.times = each.times
                    }
                    return word
                }
            });

            return {
                video_id: this.videoId,
                total_duration: Math.round(1e3 * this.player.getDuration()),
                stops,
            }
        },
        videoId() {
            try {
                return new URL(this.videoUrl).searchParams.get('v')
            } catch (err) {
            }
            return null
        }
    }
}).mount('#app')

