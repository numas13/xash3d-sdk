#include <stddef.h>

#define STDINT_H <stdint.h>

struct tagPOINT;
struct delta_s;

// HLSDK
struct sizebuf_s;
struct client_s;
struct netchan_s;

#include "common/mathlib.h"
#include "common/const.h"
#include "common/pmtrace.h"

#include "engine/progdefs.h"
#include "engine/edict.h"

#include "common/beamdef.h"
#include "common/cl_entity.h"
#include "common/com_model.h"
#include "common/con_nprint.h"
#include "common/cvardef.h"
#include "common/demo_api.h"
//#include "common/director_cmds.h"
#include "common/dlight.h"
//#include "common/dll_state.h"
#include "common/entity_state.h"
#include "common/entity_types.h"
//#include "common/enums.h"
#include "common/event_api.h"
#include "common/event_args.h"
#include "common/event_flags.h"
//#include "common/hltv.h"
//#include "common/in_buttons.h"
#include "common/ivoicetweak.h"
#include "common/net_api.h"
#include "common/netadr.h"
#include "common/particledef.h"
#include "common/qfont.h"
#include "common/r_efx.h"
#include "common/r_studioint.h"
#include "common/ref_params.h"
#include "common/screenfade.h"
//#include "common/Sequence.h"
#include "common/studio_event.h"
#include "common/triangleapi.h"
#include "common/usercmd.h"
#include "common/weaponinfo.h"
#ifdef XASH
#include "common/render_api.h"
#endif

#ifdef XASH
#include "engine/keydefs.h"
#else
#include "public/keydefs.h"
#endif

#include "pm_shared/pm_defs.h"
#include "pm_shared/pm_movevars.h"
#ifdef XASH
#include "common/kbutton.h"
#else
#include "cl_dll/kbutton.h"
#endif

#ifdef XASH
#include "common/wrect.h"
#else
#include "cl_dll/wrect.h"
typedef int (*pfnUserMsgHook)(const char *pszName, int iSize, void *pbuf);
#endif
#include "engine/cdll_int.h"

#ifdef XASH
#include "engine/cdll_exp.h"
#endif
#include "engine/eiface.h"

#ifdef XASH
#include "engine/menu_int.h"
#endif
