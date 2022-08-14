Vue.createApp({
    data() {
        return {
            step: 1,
            videoUrl: '',
            lyrics: '',
            player: null,
            // objects like {texts: string[], start: ?number, end: ?number}
            phrases: [],
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

            // Split the lyrics in phrases and only keep the non-empty ones
            const phrases = this.lyrics.split('\n').map(line => line.trim()).filter(Boolean)
            this.phrases = phrases.map(line => ({
                texts: line.split(' '),
                start: null,
                end: null,
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
        finish() {
            this.step = 3
        },
        formatTime(time) {
            const seconds = Math.floor(time / 1e3) % 60
            const minutes = Math.floor(time / 60e3)

            return minutes.toString().padStart(2, '0') + ':' + seconds.toString().padStart(2, '0')
        },
        currentPlayerTime() {
            return Math.round(1e3 * this.player.getCurrentTime())
        },
        setPhraseStart(i) {
            this.phrases[i].start = this.currentPlayerTime()
        },
        setPhraseEnd(i) {
            const currentTime = this.currentPlayerTime()
            this.phrases[i].end = currentTime
            if (i < this.phrases.length - 1) {
                this.phrases[i + 1].start = currentTime
            }
        }
    },
    computed: {
        result() {
            return {
                video_id: this.videoId,
                total_duration: Math.round(1e3 * this.player.getDuration()),
                phrases: this.phrases,
            }
        },
        videoId() {
            try {
                return new URL(this.videoUrl).searchParams.get('v')
            } catch (err) {
            }
            return null
        },
        hasIncompleteLineInfo() {
            return this.phrases.some(line => line.start === null || line.end === null)
        }
    }
}).mount('#app')

