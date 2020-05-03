let processor = {
    timerCallback: function () {
        if (this.video.paused || this.video.ended) {
            return;
        }
        this.gc.rander();
        let self = this;
        setTimeout(function () {
            self.timerCallback();
        }, 30);
    },

    doLoad: async function (vid,cid) {
        this.video = document.getElementById(vid);
        let filter = await import('./pkg/webfilter.js');
        await filter.default('./pkg/webfilter_bg.wasm');
        filter.sethook();
        this.gc = filter.GrayColors.new(vid,cid);
        let self = this;
        this.video.addEventListener("play", function () {
            self.gc.set_size(self.video.videoWidth,self.video.videoHeight);
            self.timerCallback();
        }, false);
    },
};
