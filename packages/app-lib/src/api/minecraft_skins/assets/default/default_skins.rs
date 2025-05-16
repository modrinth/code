use std::sync::{Arc, LazyLock};

use url::Url;

use crate::{minecraft_skins::SkinSource, state::MinecraftSkinVariant};

use super::super::super::Skin;

/// A list of default Minecraft skins to make available to the user.
///
/// These skins were created by Mojang, and found by reverse engineering the
/// behavior of the Minecraft launcher. The textures are publicly available at
/// `https://textures.minecraft.net/texture/<texture_key>`.
pub static DEFAULT_SKINS: LazyLock<Vec<Skin>> = LazyLock::new(|| {
    vec![Skin {
        texture_key: Arc::from("46acd06e8483b176e8ea39fc12fe105eb3a2a4970f5100057e9d84d4b60bdfa7"),
        name: Some(Arc::from("Alex")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAFJklEQVR4Xu2aP2sUURTF/RRir2BAsQyC4GJjaSwEg1iJqRSDgiBooSIWoggiYm0l2KiNiKCNhaWFlYVWVhYWfoAxZ8xZfnvyJtmdkN2dMBcO7++8fefc+2Zn3ps9e7awX48G1e/ng0rpjztHRlJD7a5zXqmQ43XOTMQQOSPrd7UAf18u10jCTRHiuhyvc2ZCTR5mPUVwW47XOaP3GQXON4ljMXK8zll61eRc/vn5RfXt3eOhEFwau0IAk7bXGe6CBDAkhGExcrzOmUgy7Bn+jgDh99fXI3B9jjd3lqFd8jKJu815Lgf3Yx3Hz7LyOZ+pmwmWSCbZJMd6lykCyymAr835TN048SSek1bq0E7i7M/ooLAeg7+X85m6pdecFkVYW9t/vn8arvOSCKxznoQpylzcJD0pkh/x8hpR2f4LB2rygvIyi1DyMoUkYfdT/VwIkF7mHd2EN4OFoggUIAlTpLkQwJMcPuxsRv7Dm/9YL/MvzySV2tMkTO/z93I+Uzd6ww8x+b++QYh18nlDJHGP6ehi3m1zIUDa1VdXKmJxcbHGwsJCjeyf9vDjg+r++3s1OYPEc/xEjpem8Q06I/u1tpyQiVuI7J9mAQR5ebi01kXwuOePHRxBWwFEXlGb/VpbTrBtBNy+e27kncAilMhThBwvLQXYkQjgpNpEgMjz5YjLIcffrgDbjgB5ywPefHujunjicHXt1GJ16+zxOm84MhzeBidE7/MmqQmqjktrK2gupfETTX2SZ6OJnH7M5C+fPFKTF5SnCIK95bJ+zNd7Qk+fXRr5m1S+JICuKdWRVArtPOvTIUqTZ6PJ2yIq2PMmr1R1FCD7UQBPXqlFSPIkndeZ5NMvT4ap8yyX6vLa5NloJkKiJG8xSiJZAJKxByxCkncbyyScqa5vg+TZaAxzhr8FYIQICn+WSSYnrzTJUoCSF+llgQ9khO8vCbcnz0ZbXl6uDIa4iauOfYSVlZUayjOMk0AdBWsp12aKUurPcfIJlH99FIL3GyF5NprJmJAJlgiXQELpzSREAQy2u8z6JJ4ilMhPJECJ4Orq6oY6izIYDKqlpaUaypMQiSgvUZKQ692HQrluqgIkoQzxbFfqyFB+39G9Qxw6s7Dhbm/CJqY+vIZiZKq2JJ0CcCm0EoChTpIlwu7DsgmYsAgmSYtjgdg/2wnVJ+kSeUbCxAKYLEl5CSTxFEiQp3xnF/KBJFP3cfgT9jzbk7jJ580vRUiejabne5EysVzzJi34fYBlT5Tk9M9x/fTREaiO4vCaFIJI8ikC01YR0FtvvfW20+bdIG6Jc5uMcFuO0XnL8wLXc7doVxKX2fu7luBW5tDP+nFMewqJfPdPqE+OM1NrS7633nrrrbfeeuuWeU9QaHO4qmcGb4Z0coODAgiTHq/7XUEC6FB14i2uWVseeU8aAX5n6GwEpACTRoAE8BLoRATw5NjwuSLPHQ2JUtoI1QapX5ezLTFXb5VJ0ALwcDUPWRkh3CmmALl7zH5Kcx4zMxL3wWqeMpfg43gR4nkiy1nHNOcxM+NROb8dYPjnNwXsk+/y48K/n/VG7iHkXgI5bMtSgCRrTztCEnm+n6c9JahPzmNmxuMzLoNxvi/QqZMJlU53mqA+OY+ZWekoLc8V85id/enVccgbOY+ZWRJUeZLvCzovQBLiAWuJsFJHhvK55pNoE3IeMzOGOkmWCJeQAowjwlzdA0yWpDb7viBB4rwZJmmSn6t/gc2+LxAcCULp+wKu/63I70QE/AMDdqWZ7rX6YgAAAABJRU5ErkJggg=="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("1abc803022d8300ab7578b189294cce39622d9a404cdc00d3feacfdf45be6981"),
        name: Some(Arc::from("Alex")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAFQElEQVR4Xu2av2scRxzF9VeY9DZYEKPyEBh8pHFpuQhEhFTBqhIiEjAY4iI2JkVIMBgRUrsypHHShGCIGxcuXbhyYVepVKTwH7DR2+hzvHs7e9KelNWt2AeP+bGzc/PefGfvdubW1o7A3z9Nq/1fppXSt/c25lKo69SRVypmf4MDQqDEwaw/1wa8f7JdMwW3RQh12d/ggKC2GfZ6N4Fr2d/g4LPvUUC+zRzMyP4Gh5xVxFF+9+Jx9fqPhzMjfGmcCwMQzax7uIsyAMoIiBnZ3+AgkR72Hv5EgLj/6ukcqc/+Vg4Z2qVZduFcI+/LgXZe5/1nWfkcT+9AYElkik1xXk/ZTfByGsC9OZ7e4QNP4TlopYR2Cvf2Hh1uLH345+V4ekfOGmnRhIO1/c+b57N1XjLB68i7YDdlJR6SDMrFz83ygVDh4ueXavGi8gImlGbZjXTBtFP9ShiQs+xPdAQvIka5CW5ACnaTVsIABjn7sbNI/LPf/uNh2b/yEKmUmXbBPvv+eTme3uGzwY+Y/F5vGHEoPh+ILpw+iS7Pc20lDEh8/etXlXMymdRcX1+vme0TP/71Q/X9nw9qcdCFZ//J7C+h/p05Kdm+M3JACMeIbJ/AAFGzPFtahybQ72dXL8/xpAZIvKI323dGDnDZCPju/qdz7wSYUBLvJmR/iTYDTjUCfFDLRIDE+8uRL4fs/7QMWDoCNFt09u3vd6pbH12pvrkxqe5+cq3OQyKD8IY5IGbfH5IamOp8aR1FjaXUf4mL2qXeBiROH4b4L69v1OJF5d0EkdmirA/hfgaz9/MXc1+TypcM0D2lOhdUMpqyXytNjNLU24BmW0JFZh7xSlXnBmQ7N4DBK8WEFO+i8z4E7r18NEvJe7mt3k2inHobQIgLdfGYUTIJA1wMzmNCiueal11wprr/JEy9DXiYe/hjgEeIqPD3sovJwStNsW5AaQZzlv0HWZJnTNLbpN4Gtre3K+ghjnDVeRtxZ2enpvIeximgjoKD1NdkmlJq7/3411wyjfBnDky9DSAGQQgsCS7RBeVspiA3APp1yl6fopMYUBJ/LANKAnd3dxt1mDKdTqutra2ayrsgF6K8TElB1NPGjaLupAZ4PvU2kIIyxPO6UiJD+Q82L8z44cfrjac9ghGmNn6Pm5GprqXgpK93GZHXU28DHuousiSYNl5GAIIlMEViDgZ5+7zuVH0KahPvkdDJAMS6KJZACk+DRM0UT3Yxf4hkShvC38nM+/UU7eLz4VcyIfU2oN/3EoWwXPOIFnkf8DIDdXH65rh9c3OOqnNz/J40wpnCSyZ42jkCRowYMeL/BrtBviXu22ROrmUfg0eeF1Dvu0XnUrjA7J9bgUeB0M/640B7CiXme3+SdtnfmWBZ8SNGjBgxYsSIYYE9QXGZw1X9ZvANkcFtcLgBYtfjdd4VMEAHq522uM4aeeTdNQJ4ZxhsBKQBXSNABvgSWPkI8JNjyLminztCmVLaCNUGKa/Lea2NK/F2mQIxwA9X85DVI8R3itOA3EEubbPneHqHC+dgNU+ZS+Q4XkL8PBFS5/VeJs3x9A4/Kvf/Dnj4538KvE2+x58mcw+htJeQejojDUixzDQRksyz/TzpWUS1zfH0Dj8+82VwnP8X6NQJIW0nO4uotjme3lE6SstzxTxm9/Y+m13EwxxP70iBKnf5f8EyBnik5Hh6RwryA9aSYKVEhvK55lPsUczx9A4PdRdZElxiGtDFhJV4BiDWRS36f0HShfvDMMUmaZ/j6R2L/l8gEgli6f8FCPE0xbbxNCLgXy/vfcHbfYbuAAAAAElFTkSuQmCC"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("6ac6ca262d67bcfb3dbc924ba8215a18195497c780058a5749de674217721892"),
        name: Some(Arc::from("Ari")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAEuklEQVR4Xu2aMYsUQRSEhYsMTARBRBRUxMsEUUMTDw1ExESDCzQRo9NQ5EBNNDMxMjAy0L/gbzH2B2giorBSA3XUfnbP7eytM7N3W1BMT/fb2VfVr2dmZ/bQoV3w5urxybtrpyYfNs40W+2rLapd4tend3bI4y0dLDwNsCkUTvH7wgCL94znfppA4fvGAM54F/H7woDdKoCCSR5v6ZCiswrcpmCSx1s62ACbkMshK+Dbi4dT3FcG5JWAy6AkPk3g8UYHn8gs7OXFo1OzrH0bIEpYjjM2qyP7tDXdry3z6R2ZbG5phoR/f7fVUG2Oe5/Hyr5RGuAEc5Y84x6T6F+fXk9RfRmjz1iUDUiDWA2OYz69w8lIgMV5rTthijdTTH6+JDQrIw1hPr3DiViAy9slr3Hh3NrajkC1BY3lktDWBpRmPs3wPvPpHZmMSz3b319tNeTsZ7+NcluC0wRXQ5rgfebTO7I0nZhPdlz7vrxNGRGxrgYdIy+HFG1DRlEBKd6Jib6+0wSK9/V+FtqINIT5DI7DG3cnyUuXrzQ8f2G9IeMJC9P2x9uNhr8/39vhkdubreTxiJ/vbzXH+fp8faoCGTc3aICF2wjGE7mUvFWV2AyJ5HeYsxhgI2WAhC/8lptJda0Anui8lBZpwJ8vD8ddAUrS5wefR/7HElhIBTABkoZkn9pKpEQaoFnzzCUlRqJEV4naHveYDfS+xhxv+ngidVZx7P6jSRspmPtOKBMTJdblafEWkEKdcI7lOi/F5Ji/j8ejzioo+MSDJ1NUnwXbkCQFpQFJinR8TSDbpX3PehrgGOqsomTA6cfPpgyw2KwK71NQlnaJjKcoGsBxHocGeJw6q6ABWQUlwSQF1QzIONKJp+C2+GQakKTOKiioxHkMsBgK4T7HSB+vK6mzCgoqsc0AihCVQE0QY9VXqhbH+kRaY15pktRZhYVxy74a0/WSaAqzQY6nCWmSYiSGvz/4Q4y/WToZkNd2ijN5/U/qju/jjbP/bMXt7e2G3s+YUjypMRpAM5JzGeA7PHPjxs3J5uZmsxU5TuaM5qyKNonrsxRLerw08yXxNIE6q8jb3DRhFvH6LA1w4rl8ZALjKLjGkgFq59rntrMBeyHFixaf54+2SmhjzjorgMLnqoBFQ+uWff8DfX1PZ/SVWF/f0xl9JTaKB6olHDgDfE1PLiK53Z4a8zvFUTxk9SMx9s+DNkFtY4NiNC88hoLfLbD/wOD6ySMTkf0HBouuAP5U9i9G/nLMPm15nF6xSANWWGGFFVZYJvCZoR+c+CkS44l8QDqahxxdQAPy8ZnIeCKv937z2/kx15CgAV0rQML9LvFAVkAugaWoAArmO4USM5ZPgUsvVbg/+K1ugoL46oxkvIT5CbAMSNEpmIYwj8EgURakdun/BTQgKdElAyg6+0ZngMXYAP6/gMx4GlASTTNGbUBWAYWXSANmJfMYDCUDupBvgmalv1/VYFNcJbUqyrHUsCfs1QC+55uVzGMwpAE0wu3ckjaANz219uhuiiTCl7TafUCOk5zZNtHZxzwGA1+Zd/1/AQ3YzQT3M4/BQEE2oSSevwtsQL7mznf8JQMcxzwGA/8v0JU0oCSaXGQF/AXuGcKOL5bNbAAAAABJRU5ErkJggg=="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("4c05ab9e07b3505dc3ec11370c3bdce5570ad2fb2b562e9b9dd9cf271f81aa44"),
        name: Some(Arc::from("Ari")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAEyElEQVR4Xu2aP4sUQRTEhYsMTARBRBRURDNB1NDEQwMRMdHgAk3ESA1FDtREM5OLDIwM9Cv4WYz9AJqIKKzUYB21v+mZ3bnb65u724Kip7vfzr6qefNnp/fQoRl4d+34ZOP6qcnH1TNNq762RW2X+O353U1yf3sOFp4G2BQKp/h9YYDF+4hnP02g8H1jAI/4EPH7woBZFUDBJPe355Ciswq8TcEk97fnYANsQp4OWQHfXz2a4r4yIO8EPA1K4tME7m908IXMwl5fOjp1lNW3AaKE5TxjszpyTK3pcbXMpzoy2WxphoT/2HjaUNucd5/7yrFRGuAE8yj5iHtOon9/fjtFjWWMPmNRNiANYjU4jvlUh5ORAIvzue6EKd5MMfn5ktCsjDSE+VSHE7EAl7dLXvPCuZWVTYHaFjSXp4RaG1A68mmG+8ynOjIZl3pu/3jztCGPfo7bKG9LcJrgakgT3Gc+1ZGl6cR8seO579vblBER62rQPvJ2SNE2ZBQVkOKdmOj7O02geN/v56GNSEOYz67j8Oq9SfLylasNz1+42JDxhIWp/fl+teGfL/c3eeTOWi+5P+LXh9ub+/r28mKrEhk/GDTAwm0E44k8ldyqSmyGRPI7zHkMSDNtgIQv7NGbSQ2tAF7ofCot0oC/Xx+NuwKUnK8Pvo7s5CmwrQpgAiQNyTFtK4kSaYCOmsg4CZEo0VWibc97LkvfY5r3Z0zv03HU28KxB48nfaRg9p1MJiVKrMvS4p1UCqWwvGjaAMZ4zPP+ztI+qbcFCj7x8NkUNWbBNiRJQWlAkiIdz4SzX5rjmI96GpAx1NtCyYDTT15MGWCxWRXuU1CWdomMpyAawHnGlgzIeeptgQZkFZQEkxTUZUDGkU46BffFk2kASb0tUFCJWzHAYiiEfc6R3t9WSb0tUFCJfQZQhKgv7hLEWI2VqsWxvpD2Me82JPW2YGFsOdbFdLskmsJskONpQpqkGIko/QbhAxB/t3ibelvIezvFmbz/J/XE9+nm2VYrrq+vN3Q/Y0rxpOZKBtCM5GAD/IRnrt68NVlbW2takfNkHtE8qqJN4nlZiiU9T0F94mnCXAbkY26aMI94fZYGOPE8fWQC4yi4i10GqJ/nPttBBmyHFC9afF4/+iqhjzzqJROypVHUu+PQecuxnULN75obNZOq+V1zo2ZSo3ipShw4A3xPTy4isXneGvN7xV1/0epXYhzfCmaJmTW/KxjNgsduwWsLHD8wuHHyyETk+IHBoiuAP5XzVyN/PeaY47i/KlikAUssscQSS+wl8J2hX5z4LRLjCb4g5UsOxo8ONCBfn4mMJ3i/96qvTWD86EADhlaAROda4oGrAJ4Co68ACuaaQokZy7fAfYsqpXHmUx0UxKUzkvHNUf7/BlgGUDBNYJ/5VIdEWZC2S/8voAFJiS4Z0GUC55hPdaQoG8D/F5AZTwO6BJcMGaUBWQUUXiINGErmUx0lA4aQK0FDqSqwGa6OvirK6lFLPYOxXQO4xjeUzKc60gAa4e1sSRtgQbnux3XA7I/mwUgifEvreg7IeZJHdJZgjjOf6uCS+dD/F9AACuzra5v5VAcF2YSSeP4usAG5xM31fRqQ44plPtXB/wsMJQ3oElziIirgH4/zgLkTkjiIAAAAAElFTkSuQmCC"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("fece7017b1bb13926d1158864b283b8b930271f80a90482f174cca6a17e88236"),
        name: Some(Arc::from("Efe")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAFK0lEQVR4Xu2awYscRRjFcw9JhLiHoMIGzCVLIIRlEQJGDDkIghA9RBBBBdkl4EEFPYm5CCrkorlo9CBBCJGQu168GTx49OYf4D+QY8tr+A1vXqom3e3uzHTSDx71dVV1zfe++qpmumsOHXoErl35svnq3W9bfvHW9Zmtel1/9875hczxRgcX7UGgROgHl87M8bEJgIv3bIAl8R6EHG90SMEeBNmPfQBy3WN/+tpnT0YAEC3BiGYDVL3KFA7VluONDoi+cfXH2exLGPUqS1mgOrXleKMDYnMvoG7RV+EoMsBFlda713kWeFBK91EPsx47/Vk6So5LnK9xtX2zd3Ouz/sX9mZ2isrgsFRK/dOfpUNO5Gy7GNpv7b7U3Pnw1ZlAXXufFOdLhHtKmZL+LB04p/L3z99o6Y7KRnwSEZTc73WeNbIZGzv9WTp8diRKO3g6Luxtbc+EyxZo1/3aH3Sv2kl7SgJJHVyLbwkcUamdWwIkRs6qFHPmIe26/+ZHt9s6dn/VaZ9gbPYBD8RaBMCdUfnT7sutGJaFBP68+0or7pOLmy1lq079yACV//7z+kygxGvpqFSdbPUjELSnP0sHsylnRM2gxOC47Jx5yBJwkTD7+md4n/Rn5Xjh5LkGvnhqp3n7xLnm8sZWS9nZP3Hvrz8b8fb9P5rvf/u1pWzqNe7557dn44tel+MlfGwCu6/PHC5elPD3nttpKTv7J3BQG+KbO5stZRMExpdgJ/U5XkJjEFTE6zOy32C4+KEZ8PXduw8FQHX7kQHMvsY6kAxw8SKzD7N/Qg5KLEFAvKi2HD+Z4yV8Se1LBijqZ585/ZAji0i6iqztXPuHjx1vnnr6REvZuRfkPQTI7ezDGJB6+pAZKlNnFRK0tXFqTlgKLvFRAdCYzgxACsB5t70tA+D9s5+YOqvIGVU29AlEBgBHyAJmPx3OAGSdszTzGYy8J3VWIaGaIQQTBAKRgkVvT/H6cKWwnGIJyGYTrIktiXAuEsw14+k6dVZRm+1SnYuHpQC4Y7X1nP1LAr1vtpcCOTgAtT2gFITskwFwJ3zWkxmEvJdrvkH6MnVW4Wsekrq1AHg2yNkkM0MAfBYR56UHANHefwhTZxUuFvvvH7abBw/uzQUCuyuPHNtoH440juxs934EgaARELXle8auTJ1VpEOiHM+6Gj07fOP03xbYpYzyDdipOrWlsK5MnVWkoL7M3/SiBMt5Nkrs7Kf6zaPPzoLAbwZsteUTpf/8TXp76qwiBfUls6fSvx6pW9Qu5nOBZ4DaUjji/fyB0oOQOqtQumvNp7Ck+qgv61qU7c674C7XIpmR/cikFJ9BKAWg1xKYMGHChAkTJkyYsKbg+Z7H3Np1MscZLfw9QR/mOBPGinz4cpsHOfGXj8/OPbDlOBMmTFgO/MWHOOR0mZekg97wrBr59mfI/wsIgE59taGN6g2Pv+4amgG8Ih91BsC+/y8gACqVAbzry35rA1/vKd5ZOoFSvQt2Upellsda/dJbJNyF1s4fPeVLglM4+0P6sTKkaBeZs579agEQEUydH5n1Ovs7aKQghOcBCSc+3l4KADOcJU+A1KUfK0MGAMGlOhcPfZY91WszT7DSj5UhhZZEy85DUNryOb4r+Xyu/f0ApdPbS+MMhq9x6OeGTk99goBjfcnnky1D6VoGAbES5ueIXf9fwPd8X6YfK0MKgl3/Y+Ci8gB0EdOPlSEF9WUGoEsgRpEBXSkxfsyNnaJdvPqkHyvD//1/QQZgkfiDyID/AMdpmCl88QvjAAAAAElFTkSuQmCC"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("daf3d88ccb38f11f74814e92053d92f7728ddb1a7955652a60e30cb27ae6659f"),
        name: Some(Arc::from("Efe")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAFPElEQVR4Xu2awatVVRTGnUcZ1BtIBk/ISY9A5PEQBA2jQSAE5sAgggriPYIGGuRIbBKo0MScqDUQEUQR5zVxZjRo2Mw/wH/A4YnvwO/yve/sc9895+npXD0LPvY6a6+z7/rWXnvfe86+e/bsID+duVRd/vrXGj9/8ctMl13X1786Ohc53tKJk/Yk0EL0+48/2IaXJgFO3qsBlMh7EnK8pZMk7EmQ/tInINc9+vlPL7waCYC0CEOaDVB2tUkcqC/HWzqB9LXvfp/NvohhV1uqAtnUl+MtnUA29wJs874Kl6ICnFRpvbvNq8CTUroPO0g7esYzuJQCFzlf4+q7unVzm8+3x7dmepLK5LBUSv4Zz+CiIHK2nQz9tzc/rO6dPTkjqGv3SXK+RLinVCkZz+BCcGofXTxdwwOVDvkEJGi5321eNdIZGz3jGVx8dkRKO3gGLtlaW58Rly6hX/drf9C96qfsaUkkNjCKbwkCUaudWwRERsGqFXLmAf26/+a5u7WN3V827ROMzT7giRhFAjwYtbc2T9RkWBYieGfzk5rcjx+t1pAum/yoALVPn3w2IyjyWjpqZZMuPxJBf8YzuDCbCkbQDIoMgUvPmQcsAScJ0tc/w30ynv9djhw4XIFjBzeqL/cdrk6trNWQnv4pD//5uxLu/vW4uvHnHzWkY9e4R99bn40vuC3HS8mxSS4/vtK/szh5QcS/eXejhvT0TyFAbYifb6zWkE4SGF+EHdhzvBSNIeK0kOez0r+zOPm+FXDlwYNGAmR7HhXA7JPo514BTl5g9kH6pygokSUJkBfUl+MncryUXFK7rgBl/dA77zcCmQfKVSCQXPuv7X2revPtfTWkZ+B5DwlyPX18T6Hf/bD7ZyXfhojQ2srBbcSScAk7JUBjOjIBSYCydt37nBh621juk3wbkjOqauiSiEwAAVMFzH4G3Ba0607Erz0p2d85ASKqGYIwSSARSVjw/iSvD1YJ68NZAtLZBNvIJslEG2G/LrXJtyFts12yOXlQSoAH1rae079E0H2zv5RIWu9Pvg2hAiDtxEtJSJ9MgAfhs57IJOS9XPMN0hfJtyG+5gGl25YArwYFm2BmFIATpB+d1hMAafffDZJvQ5ws+r+/rVfPnj3clgj0RfH63pX64UjjSM9+91OgnjQSor58z9gVybchGZCgwNPWBq8O3zj9twV6qaJ8A3bIpr4k1BXJtyFJqCvyN70gwgqejRI9/WRffWP/LAn8ZkBXXz5ROpIscJ/k25Ak1BXMnlr/esQ2r1/I5wKvAPUlaSfvZxDZLpwAlbvWfBJLyEe+rGtBugfvhBe5FqiM9KOSkngpCd52roBJJplkkkkmmWSSSUYqelLUEyKPtzzn52NvvgdY+HF47OLvCfogx5tk2SQfvrhG10Pa/R8O1ZDuD21CjjfJJJO8WPEXH0Kf02VekkrPlxzpPzrJtz99/l/gCdCJrzYzkpD+oxN/3dW3AnhFvtQVALr+v4AE8EOGc//RVoCv9yTvKJ1AyZ6EAbZsgZaJ2oxncJlH3Im2nT9mybe1EPZTpFEkIEk7yZz19GtLgABhbByZeQKEjGdwSUIQzwMSTny8v5QACGabCaDNeAaXTACESzYnD3yWvdRz5vN6NHtAEi2Rlp6HoPTlc3xXKBGul67dln3Jp7P4Ggd+bujw0icJHmQfUC19kXw6C2RFzM8RF/1/AT90+iLjGVySEFj0PwZJKA9Ad0LGM7gkoa7IBHRJwqgrYFGIhB9x5/l+G/DPeAaX3f6/IBOwCHlPQsbTVf4Dv55a+XG6bwMAAAAASUVORK5CYII="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("226c617fde5b1ba569aa08bd2cb6fd84c93337532a872b3eb7bf66bdd5b395f8"),
        name: Some(Arc::from("Kai")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,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"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("e5cdc3243b2153ab28a159861be643a4fc1e3c17d291cdd3e57a7f370ad676f3"),
        name: Some(Arc::from("Kai")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,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"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("7cb3ba52ddd5cc82c0b050c3f920f87da36add80165846f479079663805433db"),
        name: Some(Arc::from("Makena")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,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"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("dc0fcfaf2aa040a83dc0de4e56058d1bbb2ea40157501f3e7d15dc245e493095"),
        name: Some(Arc::from("Makena")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,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"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("6c160fbd16adbc4bff2409e70180d911002aebcfa811eb6ec3d1040761aea6dd"),
        name: Some(Arc::from("Noor")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAEsklEQVR4Xu2aTWoUQRzFszGEgJqETFDUEDWJCiIGVCQuXCgo2bkRv3YKHkAXuvUGbrxC8Ag5hGcaeQNv+Oc3Vd1d7aR7xuTBo6qrqmvqvfro6q5ZWKjBvUsXho83V4cKn+8MRnGFTndeLmR9c4eUqBim0v5bA2LPx1D8sb99jE5nfXOHlAE04r83IAqXaBpxKgyQ8LcPto6FNuDni70JA5R2Kgw4/PBsJDaa4Gvlsb65Aw3glJDIKrK+mYOEpFb1VNwGsHyqLMO4fsTpw/Z0DjYyXt9cWx7F1ZNHnw8mQuWpTFUdDJnP9nQO9lCMex5L7J/v70dhpOd/7FHGaQBDtqdzRMESKepawiz09+v9EXPXKqt7fD9NcOgRNZMGuKctxtcSKry5e20sWHHBJqhsNM3XFM5wZgxQY+Jw13Uc6nXUfb4nTgsKToVsT+dwY+Kzm7TQL09ujRiFk67HAmVwDJnO9nQON4o7OW5oUr3ufN4nUmguZHt6x+Dl3nDz3dMRFb8/WBreXl0chyxPcN7HuPJU787XV0kqj/URv26sDM3YGSzXGhIdGcUrZHkiGqB4HPo2IGWC01kfQQMkXiOO5VqDBrQZAaanQ0yz0BxZH0EDZnIEcEH09TQMOLwzmO4IYAOaMA7bT2vnJph6miiN5ZpQgslvG4uVVBnqzILiKLCO8YfdaMUXl5bHI2Bl/fJEPu/JkeJP3IDc4hTp6VFlgESLgyvX58cAik+ZwDUiLkputOJRvEYD880qEywm0vfkqPwiAy4+3J4QlROrsqQFRCHRAJEGpO6hEb6OZtG4FJ1PnVlQUCljo6IBEu1pIDI/xZRoCmxK6syCgkpJA9wLNsDTICfQPNq7Oi7j+NwZEBlHgKdAjhIsquHxuhMD+LwuJcW40XFR9dOC5ZoYwN9rSurMIt7EN76q116Tzosa5h4hEq/Q0yPFaByv2Za6NjmfOrM42F0fkh8fbY3I9BS5R2DP5x6nTUnhFh+32QyLDND+nrQ4pqfIBosUzGuS+45YnuJpQgxbjYB5g1+3FTLvVEDCZ+ZQpQ/YAI0C5p0aSLxMYHoviF973DssY3joRtZ9PGUdgn9nd+P8Mbo+lu8MTQ2IaV7QnMcTpVjWcL5EO03xqns6QRsDKNrDO1XWyBmQK98Z2hggKC1FlovodajnUNfwuvwSTKueBe6+ShnrqhNYl98LKKiUsa4m4pqUifCboanPXQ5TdDnWkwX35aVkfdMGX52bkvWc4QxnOEMn4LlC6eFqXMjafODQU6bXLTEPU6L4Jsfr0QCd+pZ+4pLw3I6zE9CArkeA3yVmxoC2I0CbmjYjQOI7fU/gxqgJvYtUnJ/ITe7wTOf79/lNoK73p74+UBwF1pHC2xgQ43UCvUgyvTUoiO8KzBc9PfowQKjLL0KV+JQJXCMovM4ArQ/RAGKqvdsEOvaiqJxYHqyKFC7yxYasMqC4d9lbpaSgUlJ8U1JHa1BQKSmolBTWlG4/p4jIbwikyowN4AJVSgoqJYU1pdvPdE6XHMcG/Ct4PN0V2Y7eEBvFE9x4kltC3k/OlAH8v0Dp/wskJh5zxzP+FF2O7egN/L+AWPL/AhpQJf4kRsBfkTxADPB27yIAAAAASUVORK5CYII="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("90e75cd429ba6331cd210b9bd19399527ee3bab467b5a9f61cb8a27b177f6789"),
        name: Some(Arc::from("Noor")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAEu0lEQVR4Xu2awaoURxiF3XgRwUTFkYhG1KhRCCEXjASzyEIhwZ0bMepOIQ+QLHTrG7jJK0gewYfIM42chjOc+3XVdNc4t2fGew8cqrrq75o6p6uqq7vnxIkB/PjNV/Nfrp6bK310a9bllbrcdbWU7e0cSqIyLZV9sQbklc9UfPfg5gG6nO3tHEoG0Igv3oAULtE04kgYIOF/3rt2ILUB73/f7xmgsiNhwIeXDzuxaYKPVcf2dg40gFNCIpeR7W0dJKS0qpfyNoDxpVimuX7k9GF/Jgc7mcffnT/d5XUlP/71uJeqTjHL2mDKevZncvAKZd7zWGL/f/uiS5Oe/3lFmacBTNmfyZGCJVLUsYRZ6H9PH3SsHStW5/h8muDUI2orDfCVthgfS6jw7IdvF4KVF2yCYtM0H1M4060xQJ3J4a7jHOpD1Hk+J6cFBZdS9mdyuDN57yYt9O9fv++Ywkm3Y4EyOFOWsz+Tw53iTo4bmtJVdz3PEym0lrI/G8fsj/351ee/dVT+p9mp+Z1ze4uU8QTnfeZVp3Zv/fOkSNWxPeLD3dn83xtnF+RoZHwzJDqZ4pUynkgDlM+hbwNKJric7RElAyTcI4/xzaABq4wA053KMgutke0RJQO2bgRwQfTxugxIEz57BLADY5jD9vX5kz2W7iYqY9wYWjD55uLeUjqOenugOAocYv6oO6383qnTixFw9sKlXj3PqZHCD92A2uKU9PRYZoBEi7PL13fHAIovmcA1Ihckd1r5FK/RwHpzmQkpJOnzanTbowz4+uebPVE1sYolLSCFpAEiDSidQyN8nGaRFE4DROrtgYJamT+WBki0p4HI+hJLoimuldTbAwW1kgboR9MAT4OaQPPj/pVFjPM7Z0AyR4CnQI0SLKrDeTyJAbxft5Ji3OlcVH23YNwYA/h7raTeHjKYT3zcZpZIx0UNc48QiVfq6VFiGsdj9mVMvzKGent4fPvCnHx1/1pHlpfIPQKvfO12OpYUneJzq810tAHa35MWx/IS2WGRgnlMct+R8RReMiHT5hGwi/Ajt1LWHQlI+NZ8WNkEbIBGAeuODCReJrB8cuTbHl8Zxhgetsmhl6dsw/Bv3b545gDdJuMnwVgDssyLmev4RSljE46RaJcpP3TeoWIVAyjaQ7sUm6gZsOycQ8cqBggqK5FxxMaGeg1DHR+qb8Va2uLuq5XZ1pDAofqNgIJamW2NETcmJuEnw6RedTktMWPYXg/cl7eS7a0bfHRuJds7xjGOcYxDBb8rtH5czQWsW9XxkoPxhO4yG90O82NKih/zeZ0G+KuvTWA8IeG1HeckoAFTjwA/S2yNAauOAG9qWkeAxE/6jMCN0Rh6F6k8X5Gb3N2RjuP7gLFXf23rBMVR4BApfBUD3JeW9wFeLFneDAriswLrRU+PTRogjI1bimXiSyZwjaBwM9cB0uuE4tiftVzVFuizF0XVxPLDqkjhFj+GJQOaryqvVispqJUU30rqaQYFtZKCWklBreT0EPN9QIkZ01vEWklBraSgzyWnyRA5IJrhndqmyP5MjuwMv97mV9wW8vwat8IA/l+g9f8FEpGfuPl9v0bHsz+Tg/8XEFv+X0ADxohPE9ifVnwCzC7McvDpKlsAAAAASUVORK5CYII="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("d5c4ee5ce20aed9e33e866c66caa37178606234b3721084bf01d13320fb2eb3f"),
        name: Some(Arc::from("Steve")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAEwElEQVR4Xu1av2sUURgMqCABQQVBBK0SCdooMQQD5jSFkNgpKdIEwSZoZ6GYJohNUmlhqrSxsUlhYZM/If/TmdncrLOz3+7drcneXtyB4f36bvNm3vd2l32ZmOiD+3eudMHZqWtJyTrbn189LqVfb+xAwZ17N1LxqLsBH17MZniuDFDBXn5b6+TEkxjz640dXLSmP8r/wgDd+74VILJoC5wLA3TFNRsgHqRQv/mh7+v64vkwwO8BSoiE4KLSrzd2UNF+QwQpdnn6aobs9+s1DpraFDZ183Im5ZkFSjVBf6PX0WtEcSh9PrUjmiBIIVpX8V7yd5Foxvn10efzqR0qwifMvr03y91fn9a7v7+8S8qf79eSvkfT13OxINvs0zpNYJ/Pp3ZwQjpRXUkIhWAIp3gSY5Go6JqsM55tn0/t0ImxxCMM4iACQoGXT3ZS0agDaCMGffhNdC0aGhnUCAN8hShIyRV/+/RBQs0AJUX5yuv9QQ1B2+dTOzgRlFx5PsO/v15MX2jwaPu4NJdw6fZkJoaPPGaCXtMN8LbPp3bwrU3FuwkoIRomsKRoGqS/5fX0zfDurckM2e/zGTkWHu50lfPz8wlnZmYSenwOR0fp9lBj0Ycxv77TL5fDwUE34f5+Znt6WGX4hCicRnh8DsciVbyacNoGqNEeVhk+oSoZcNYGTB4epgY0MgPOdAscC9ct8M8Z8Gxurwvij6N8vvAjJftYkjp+cXc3Q0/PVHxv4kXxYKafQpW9uLS9tZWlxg4KChzUAB0HMdlL29sJU0E9oUxP74/iMyK9j/2RARsbJ6QBHB8UZQI9O3wchJALm5sJUdcURel9Hu8muFkZ8WJGMlZkADgoygQWGaB1iiH7CfL4UJi0c2mupHg1gRwULpDtqE/HSBeUWXEzQFc/MiC39/f/3uh4s9N6RI67zkK4Ab7CkSHa9pRODegJcQM83jMkFd9r66OON1Rtq+hKj8XIAN8GZeMURXoK66p6bBTvv3VhKj4ygKXrLASFFQnsZwAmqWKSDNAVZTb0BHl8aIDED2oAykoZ4OmtwpUeR1KQiqNwF8cYj8sYIOIR4yvrBqgRlQxYWVnpgn5zczLOqaIiM1Sgx6YGiGgVD6rgoizQLTC0AXi97XQ6GVGrq6s5oYjxWNR1i6hhmjk0gsI8RrePxiATIwM0K9wAlq6zEHzHpzgXSOFRHOpqAKnbiCVX1OP9XsMY9pcJ1/3vRrjOFi1atGjRosX4AR9Ai5h5fT44+ViajrVo0aJFixYtWtQN/5Q29OGqfELTDyEe1li4AUMfr4sBmZPlcYEbUCUD+JrLDBhrAypnwHFZ6airbujXX//I6WM6zpjcgaefEbDtbAoGEcls8DEwPOJWA9wEtpsCNwClG6Dj/QxID0vcCK83BS7QRfq49iUGlJ3xl5HQTOmZ498NnKdqoItnO+rTMdLP9/XG5wce2u/zGBmKBPtq67jGu1h99Hm7kY/FSGC/LaB0A4pEs5+lz2NkcIEoIwMic8DofM/Fq/DGZYCnv6+09kfbJNrnkXDv93mMDDxJpsAi+nE7WSY+Et44A3hcrqKG+f+CyAAV7Qaw7fMYGaL/G1CBFB7FoQ5BRfeByAiO+zyq4g8lK5z2I+oYkQAAAABJRU5ErkJggg=="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("31f477eb1a7beee631c2ca64d06f8f68fa93a3386d04452ab27f43acdf1b60cb"),
        name: Some(Arc::from("Steve")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAE5klEQVR4Xu1aMWsUQRgVVJCAoIIgglaJBG2UGIKCOU0hJHZKijRBsAnaWSjaiNjESgtTpT0bmxQWNvkJ+U9n3ube8ubNt7e5xOzdxX3wmNmZb/fmvflmdtm9M2dqcOfmxR44N325KFnn8ecXDwbSrzdxoODO7auleNTdgHfP5hKeKgNUsJff1zqZeBJ9fr2Jg4vW9Ef5Xxiga9+XAkRWLYFTYYDOuGYDxIMU6psf2r6tL54OA3wPUEIkBFeVfr2Jg4r2DRGk2OWZSwnZ7tcbO2hqU9j0tQtJyjMLlGqCnqPX0WtEcSh9PI0jGiBIIVpX8V7yvEg04/z6aPPxNA4V4QNm2/ar5d7vD+u9P1/eFOWvt2tF2/2ZK1ksyGO2aZ0msM3H0zg4IB2oziSEQjCEUzyJvkhUdE3WGc9jH0/j0IGxxC0M4iACQoHnj76WolEHcIwYtOGc6Fo0NDJoLAzwGaIgJWf89eO7BTUDlBTlM6/7gxqCYx9P4+BAUHLmeQ//8XKxfKDBre390nzBpRtTSQxvecwEvaYb4Mc+nsbBpzYV7yaghGiYwJKiaZCey+vpk+Gt61MJ2e7jGTke3vvaUy4sLBScnZ0t6PEZ9vbK5aHGog19fn2nXy7Dzk6vZLebLNHiN44LHxCF0wiPz7AvUsWrCSdhgO5H+B0PHxo+oKNkwEkbMLW7mxkwVhlwoktgX3S0BI6cAU/mt3sgfhzl04c/S7KNJan957a2EurMYFCl+P7Aq+LBpF2FiuCSbPv0KaWfUwcKPKwB2g9isOc3NwuWgvpCmZbeHsUnIr2N7S4ehOiNjQPSAI2rwyCBnh3eD0LI2Y8fC6Je/nC/9DaPdxPcrEwQ2T3ImNAAPacOgwRWGaB1iiHrBHl8IoyiVISnuJPi1QRlHVwgj6M27SNdUDLjZoDOfmRAtva7B5scyQ1Vy4h6juvN4Ab4DEeG6LGndGlAX4gb4PGeIaX4/rHe4nTH1zYVrfHF5luHyABfBoP6KYr0FNZZ9dgo3s+NRKkRkQFad70ZKKxKYJ0BGKSKKTJAZ5TZ0Bfk8aEBEh8ZoEaoASg9xvVm8PRW4UqPIylIxVG4i2OMxyUGiHjE6Ky6OBqgRniM682wsrLSA31zczLOqaIiM1Sgx5YGiGgVD6pYFxeZ4P2uNwMebzudTiJqdXU1E4oYj0Vdl4gapplDIyjMY3T5aAwyscoAzQw3QGNdbwY+41OcC6TwKA51NYDUZcSSM+rxvtcwhu0qxkWCfmv0GNfbokWLFi1atJgc4AVoFf3xmU+dSUyLFi1atGjRokVT8FdpQ39c1XeE+/S3QB4+dnADhv68bgbwTc+hX3GNGm7AUTOAj7qaARNpwLEyYL+uGTCWS0Df/vpLTu/TfsZkHzODbwS6JDKOGocRyWzwPjD5wqsGDDJB20YNNwClG6D9dQaUH0uqBLsho4YLdJHer22FAXXf9+uoWdI3xt8bVPGfGOjieRy1aR/p3/b9Q4d/8FCiz8fTOKoE+2xrv8arGL3f687vt8Kxui1GAuuWgNINGCRc+1j38TQOF4gyMiAyB4xS3YW7AUofT+Pw9PeZ1vZomURr3EUOygofT+Pgl2QKrKJ/bifrxFcJHxsD+LlcRQ3z/4LIABfsx9rm42kc0f8GVCCFR3GoQ0jVPuBZEfX5eIbFX3srPNN8aUvJAAAAAElFTkSuQmCC"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("b66bc80f002b10371e2fa23de6f230dd5e2f3affc2e15786f65bc9be4c6eb71a"),
        name: Some(Arc::from("Sunny")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAFG0lEQVR4Xu2az4odRRjF8wYiuHIhOoIuRAYiMkNEREWYIRLjahJGmIUaYVAkszAQgwHJMBsRwexCiDAbxYVZj27i0ieY5/AFWk/DuZz8Ul33L/dPpg8cqrq+6rp1Tn3dfW/1PXduCNbW1pr19fVmc3Oz5fb2dluqTbGHn29UyfFWDhadwvOYgkmOt3KQYGeAV911kYJJjrdy6DNghAx4dPDOE8LdxvFWDqNkgMSmCXnM8VYOpSeA6Qw4vXNxINpU21NhQD4GZQAvB4stUTGOt3RIcbmyKlN8zYSM51i+T7heOpfzmTs8SU7OE0zhXQaUhHYZwPM4n7mjtMopykzhKc5kJrjNY/FzlsaAnffON+aVDzYGLE2cxmSfPDfHFC+/9XqRinE+c8d3V99uzH8f7LXUDUzHEqa68H/Xwc1NdUF19VFf1X1+jjmMnM/c8cPuhcbMx5eOVVpUF9Xn4NKbjz0Oc0zz9sdvPEa3cz5zR+kLDB9nFnvj/RdbZqbwsVca78G1MpfiMenJkvxSw5VPwSl6XHI+C8e7B382l249aqn63t5es7W11VJ19idu3jvfrH/6xxPZsfvtc41ir175pUqOR2icL398uR0vs5D9JgYNsHiT/QlNTAbwUvCkJfL5i3eLHMUAjSMjVUr4zDOJBoybAZpYyQCv3LQGzD0DJDrJ/kReAnkP8cox5UmOR8w8AzRZfbDKGt2HBmklPvrqmUHpDNAkvTqq58pl/2zPUmOo/snNF5oPv3h2UGa/Wow6O2FxF67/PRArYaKPFUuTMs7JizbGK0VxLN0vS4+9f/RaK9Bl9qvFqLMTNIAr7FhmwNWjf1p2ZYDaLZDi2J9muFxIBqgcZoCYcYkgT05OBnVNhuKypBk+z2NL2CSkzk6UDPAKD7sEbBCZ/dIIiaZZZI6tMShsVFJnJ2yAJ50Csy0vgezXdX5eJmrLrKBZN47uDz7T2eUMyzs9v1zVYtTZCa7euLQBeQ+RcApxvzzH/WWA6qVz8llf+jqeBmSMOjtBQeOyZIAEZAZYlDOEN1XXLdznqKRoGmATJjaAqZMDsd2xrJcMKKWySQN8fp6T/Sm6ZADnPbEB/ICuD0rSgExlZ0GmdM0AmqcY5zJsTmMbwB87/s6fvwHIjPNSyNTmypf6cvXz0lGMwi0+r32WYxmguzN5enraku2lODOAokttvAekaB5TPE3IcqIM6NGjR48ePXosLa4dvdQkD3/bGLAW4zgrC+32JLWn8M29V9qyFuM4Pbrw2Z2HDXn9p79asr0U53iTQPsBbJsbdm/92pAWyvZSnONNgu9//n0m46wszrwBX9++e7YN6PE0IXd3VM+dpVFeruaLk/39/ebw8LAl+y0taAC309ifyFdpOzs7rQnHx8dDz1sa0IAznwHeQDXZn1i5DPCmqDdAu5iboGkQ3/BmBtRinMfCwF3h3AX2Md8DZJzv+DMDajHOY2GgAVxhx7q2wWurXItxHgtDGsCXHF0GOK5jvtYelf782p5ALZYapgIN0DFfjdOAvAQobFT682t7ArVYapgKNsDXN69xMw3Ifn7FldTd39RTQMw29eE8JgZ/6/P3fokZl7BpKHF+rWWh+fwn3Yc6JgZ/6/P3fokZp6BxyVW2eD37Kd5t6kMdCwNfT+cLSrY7lvVMbaZ+yRzHOY+FgeJKpAlJC8oU58qTS5UB/LHD/w+UmPHS6tZMmPk9YFrwvwP8/0CJGZcoZwDF+5rn/WCWGfAfZVbEuZ74z00AAAAASUVORK5CYII="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("a3bd16079f764cd541e072e888fe43885e711f98658323db0f9a6045da91ee7a"),
        name: Some(Arc::from("Sunny")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAFJElEQVR4Xu2awYscRRjF8x+I4MmD6Ap6EFmIyC4RERVhl0g0p01YYQ9qhEWR7MFADAYky16CCOYWJMJeIjkk59VLPPoX7N/hP9D6Gt7w9jfV09O7Y+9Mth88qrq+6m/qffVV90zVnDvXgqWlpWp5eblaXV2tub6+Xpdqk+3JlysTSX8LB4tO4XlNwST9LRwk2BngWXddpGCS/hYOQwZMkQFPd94bE+42+ls4TJMBEptByGv6WziU3gCmM+DwzsWRaFNtz0QA8jWoAHA5WGyJstHf3CHF5cyqTPGTgpD29OXnhOulezme3uFBcnAeYApvCkBJaFMAeB/H0ztKs5yizBSe4kxmgtvsi58zNwHY+OB8ZV75aGXE0sAZmOyT96ZP8dN33ixSNo6nd/xw9d3K/OfBVk09wHQtYaoL/3UdPdxUF1RXH/VV3fenzzZyPL3j7uaFyszXl65VWlQT1Wfn0ttHXofp07x9+a0jdDvH0ztKX2D4OrPYGx++XDMzha+9kr8H18qci9ekB0vySw1nPgWn6K7keE4d7+/8UV269bSm6ltbW9Xa2lpN1dmfuHn/fLX8+eOx7Nj8/oVKttev/DaR9Ed8/dOrtS+XzEb27wwGwOJN9ic0KAWAS8EDlsgXL94rcpoAKIjy5dLiZ5ZRDEDXDNCgSgHwrJ00AL1ngEQn2Z/IJZDPEM8aU56kP2LmGaDB6oNVTqL7MECahU++eW5UOgM0QM+K6jlr2T/bs5QP1T+7+VL18VfPj0r2pZ1+qHcMFnfh+l8jsRIm+lq2DFLaOSDRgfEscVAsOavOItW3996oxblkX9rph3rHwABwhm3LDLi693fNpgxQuwVywOzPYLg8lQxQ2RYAMe0SQR4cHIzqGggHlSWD4fvsW6JOQuodQykAnuG2JeAAkdkvAyHRDBaZvuWDgrqSesfgAHjQKTDbcglkv6b7c5moLbOCwbqx9+voM51dzrB8wvPLlTnJTr1j4Ox1pQOQzxAJpxD3y3vcXwFQvXRPvuP5dZyvwFIf6h0DBXVlKQASkBlgUc4QPlRdt3Dfo5KCyMyCYwWAKZNO2G5b1ksBKKWyyQD4/rwn+1MQWRrfsQPAm5s+JMkAZCo7CzKlJwWAwZONY5lmXJ0CwB87/s6fvwHItHMpZGpz5kt9Ofu5dGSj6BSfa5/l1AHQ05k8PDysyfaSnRlA0aU2PgNSNK8pvBSELDtnwIABAwYMGDBgbnFt75Uqufv7yhG22elv4aCdnqT2FL67/1pN1dvs9DeA+OLOk4q8/vOfNdlestPfcaD9ALb1hs1bDyvSQtlestPfcfDjL49m4mdhceYD8O3te2c7AAOeJeTujuq5szTN4SoPTnZ3d6vt7e26FNl/7sAAcDuN/QkepW1sbNRUEPb391vvP3UwAGc+A7yBarI/sXAZ4E1Rb4A2MTdBM0A83WUG0M4DVo6nd3BXOHeBfc1zgLTzfJ8ZQHvapjr//7/BAHCGbWvaBucML3QG8JCjKQC265rH2V3Zth/QZqeezmAAdM2jcQYglwAFdWXbfkCbnXo6wwHw+uYaNzMA2c/HW0k/+fMNILJdfTmezuBvff7eLzHtEnYSSpiPtCwy3/tNdF/q6Qz+1ufv/RLTTkFdyRm2eL3zKdq0TX2pp3fwaDoPJ9luW9aZ1pn2TUsg6xxP76C4EhmEpIVkanPGmzgXGcAfO/z/QIlpz9TPGaZYcmbPgJOC/x3g/wdKTLvEOAMoXmvd6730TJhFBvwLF5puRHnAy2sAAAAASUVORK5CYII="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("eee522611005acf256dbd152e992c60c0bb7978cb0f3127807700e478ad97664"),
        name: Some(Arc::from("Zuri")),
        variant: MinecraftSkinVariant::Slim,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAEw0lEQVR4Xu2asYsVVxjFNdgasAmCRMiSRMXCBBZsXpRYSdhmsbOxtNFyOzGk0DKFCUIgTVKk0lIw5B+QNGkD+VvWkTNwlrO//e6+nd3ne2/MHDjMnTvfu++ec787M+/ed+rUHHx85nS38cm57sK5s53KPrr83def9bx348o+up7tjQ4pvDLCQtOErGN7owPF65hZkWIrsr3RIUVz9JkBFdne6JDzXUZwGlAwyfZGB4qnCRRMsr3RIef+1U/PH7gXWOgH/RTwqNsAZgDFpwlsb+1gIZzbeaPjNOC1jMljVcfPsj9LR4pj2UeN5sPbX3WP71zvqbLqKKwyJ9tKs/0Z9mfpyM4mLTqFk75uM9IICs1jxrE/S0d29Nn9rV7YUcRXJuhcbaRwiqcp7M/S4U7qhtUSL3x78aO9c5UFx9EEtVVNBU6XtTAgO2/RaYCzoqKupQH+bJpIM9KEtXhK+HHlRxcFJXdml3v6PGPNfARWbTKW/Vk6+OZGZmYkKXYeHcvPsD8rx+tfdrrkxsbGPjKe+P6Lze7PH58fEKw6XXO7T+/d3kfXsz3iv2tbnfj3l7e6J3dv9vzp4fbczx0ZNGBzc3NPvMqMJ9KAHH0a0CLbIyz+98uzPgsXnkm/PbrfJRdtwL+vf96jRWcd2yPeewZUBiQZT9iAvEeo/D4MWEgGqAMSquM/L5/0dGd8XsWYSkUJE925N3+8OHDHlwG+7nh9VtS1irr21+ff9JTgFO9pkG05TqTOJnL+UVwa0IrRl2Un3FGZIFqMO65O0gALE11n4RaVZce4HX6/4qiziUoYMyBNYFlflp2uRirFZHwVm1Q9jTBbBrh96myiJf4wE5I0IEczhWUGHDbCFEqRRyV1NpEGULyu8V6Q8Sp7NFIAhdAQm1AJzxiR95J8i6zo69TZBEVRcEu8aTFpQiU8syGnAGNTvMoUztdwCvc16myiJUzUnX+eARZDMht4vWIVn2Jdrl7HWU+dTfC5P5Q2JsmplFNqKCvRVRZkvUidTVDQUD64canJH3591ZP1Q0jRpt4AbYCpuqUbQHIEScbPYyW8Ep8miNTZRL7ni7PZrNve3u6Pouv5q9BMUSrzHpGkCT7n9TzPdNdR7/1Oe6c+j4N+G1hoim2dV6wE5ZxXfb5TMJ4mkTTAP34sWHU+WvygDJgwYcKECRMmrC20qvR2d7ckl9AcqyPbmTBhQg3+Zh9Ktjc65C+v1i+yvMY6tjc6cESHku1NmPA/h+8PJleQGE/kErkXN0a1wEEDhm6vayncJnjnVys9jFtbcFFzqAGjz4DKgCH/L0gDRpEBSvNc4OQiqN8bWougSvnc3Z23VeZz9mNlyPluAyiSMUnu6npLjMKzrOvsx8pAYSozA9IElisDPNIp3PuFPrIfKwNHNQ1omZCkAUelv59rAlwzMP2PlYWvF6QBfE2mCVW2cH8/t7pbVAz7sTJQVAo+TLxpQRTunR7S9ezHytASRrbiaEBug1XCvVXGfhwbTNeh5HN/KC0wDeB+YEXqODZyhKoRy5HLo8sUNJRM8dzjbxmw0DdDjuhQUtBQ0gCbUBmR/w2gjpXhpP8vyPTPe4F+B9CA3BZnP1aGk/6/QMK8v5/iMwMy9W0C+3FcvANG/3mGjdf2WwAAAABJRU5ErkJggg=="
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    },
    Skin {
        texture_key: Arc::from("f5dddb41dcafef616e959c2817808e0be741c89ffbfed39134a13e75b811863d"),
        name: Some(Arc::from("Zuri")),
        variant: MinecraftSkinVariant::Classic,
        cape_id: None,
        texture: Arc::from(Url::try_from(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAE10lEQVR4Xu2asYsUSRjFvePSEzY5DkTB5TwVAxUWTMYVjeQwGcwuMdzkNtxMPC5YQwMVQTDRwEhDweP+ATExFe5v8fp4DW95++vqmS12bae1Hzyquvqb2nqvvurpqdpjx5bg+A/fNes/rTUn1n5sVHfp+m+XT7e8s3l+H93O/kaHFF4ywkLThGxjf6MDxavMrEixJbK/0SFFc/aZASWyv9Eh17uM4DKgYJL9jQ4UTxMomGR/o0Ou/Qsnf+48Cyz0q/4W8KzbAGYAxacJ7G/lYCFc2/mg4zLgvYzJstTGz3I8gyPFse5Ss7l981Jz7/aVlqqrjcJK5mRfabY/w/EMjhxs0qJTOOn7NiONoNAsM47jGRw50Idbt1phBxFfMkHX6iOFUzxN4XgGhwepB1afeOH6qe/3rlUXHEcT1FdpKXC5rIQBOXiLTgOcFSXqXhrgz6aJNCNNWIlvCX9d+auLgpI7s3MtfZ2xZn4FlvpkLMczOPjmRmZmJCl2GR3Lz3A8Xxxvn+40yfX19X1kPPHnmY3m7wdPOoLVpnvu9/6dm/vodvZHvP/1RvPvxVstVddk7P5+reWj7fnSzy8FDdjY2NgTrzrjiTQgZ58G9JH9ESn+xbnZXjYeWUY9v7vVJI/agI9vH+/RorON/RE24LNlQMmAJOMJG5DPCNWPyoDSEjhUBmgAEqryw+vdlh6Mr0sxptJQwkQP7N3LV50nvgzwfcfrs6Lulah7//xytWUKt3gvg+zPsSb1dpDrj+LSgL4Y/ZEcgAcqE0SL8aA1QBqQwtxm4RbkuvtxnPviGPw56u2gJIwZkCawrj+Ug2aK5uBNx5dik2qnEck+A/w3VFJvB33iF5mQpAE5myksM2DRDFMoBdaSejtIAyhe9/gsyHjVPRMpgEJoiE0oCc8Ykc+SJN818qFrUm8HFEXBfeJNi0kTSsIzG3IJMDbFq07RfBtNIzLGJfV20CdM1JN/mQEWQzIbeL/EUnwKTeGlV3K2H8gAfu/X0sYkuZRySdWSgnmdWcC4QQz4Y/NsL/969qYl22tIwUm9/dkAU22DGkByBknGLyNFW3hJfJpgI6i3g3zPF2ezWTOfz9tSdDt/FZopSnU+I5I0wde8n9cWnVmgd36nvVOfpWIOZICFpti+6xJLgnLNqz3fKRhPk8iSAf7hY8Fqc2nxB86ACRMmTJgwYcLKQrtK/3361Etuo3knSvdUsr8JEybsB3+z15L9jQ75y6vvF1neYxv7Gx04o7VkfxMmfOPw88HkDhLjCW6Re3NjNBscNKD2eF1b4WmCT329I8T4lQM3NWsN4IHK6DKgZEDN/xdwCax8BijNc4OTm6B+b+jbBFXK+yDT6c8s4HGZqWuOZ3DkercBFMmYJE9zfSRmE1I4S8VxPIODwlRnBqQJrJcMyFnO2c9zQ5ccz+DgrKYBfSYkaUAtuR/A/YIk9wqOZL8gDeBrMk0oZQvP9nm+v4iK5XgGB0Wl4EXiTQuh8DwRYnuWHM/g6BNG9sXRAB6DUTzvczzVYLrWkt/7tbS4NCBZaktSTzVyhkozljOXpesUVMucYZHn+4sMUCz1VIMzWksKqiUNsAmLjMj/D6CewXHY/y/I9M9ngc/3Swbk0TjHMzgO+/8FEuSz/RTPDEjm/wdwPLX4H8bzDGXhwa6jAAAAAElFTkSuQmCC"
        ).unwrap()),
        source: SkinSource::Default,
        is_equipped: false,
    }]
});
