#include <jni.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "ms_toollib/board.h"
#include "ms_toollib/probability.h"
#include "ms_toollib/zini.h"

// ─── Video FFI declarations (implemented in Rust static lib) ───

extern void* avf_video_new(const char *filename);
extern void* avf_video_new_from_data(const unsigned char *data, size_t len, const char *filename);
extern void  avf_video_free(void *ptr);
extern int   avf_video_parse(void *ptr);
extern void* avf_video_data_ptr(void *ptr);

extern void* evf_video_new(const char *filename);
extern void* evf_video_new_from_data(const unsigned char *data, size_t len, const char *filename);
extern void  evf_video_free(void *ptr);
extern int   evf_video_parse(void *ptr);
extern void* evf_video_data_ptr(void *ptr);

extern void* mvf_video_new(const char *filename);
extern void* mvf_video_new_from_data(const unsigned char *data, size_t len, const char *filename);
extern void  mvf_video_free(void *ptr);
extern int   mvf_video_parse(void *ptr);
extern void* mvf_video_data_ptr(void *ptr);

extern void* rmv_video_new(const char *filename);
extern void* rmv_video_new_from_data(const unsigned char *data, size_t len, const char *filename);
extern void  rmv_video_free(void *ptr);
extern int   rmv_video_parse(void *ptr);
extern void* rmv_video_data_ptr(void *ptr);

extern void  base_video_analyse(void *ptr);
extern double base_video_get_rtime(void *ptr);
extern unsigned int base_video_get_rtime_ms(void *ptr);
extern double base_video_get_etime(void *ptr);
extern size_t base_video_get_left(void *ptr);
extern size_t base_video_get_right(void *ptr);
extern size_t base_video_get_double(void *ptr);
extern size_t base_video_get_cl(void *ptr);
extern size_t base_video_get_flag(void *ptr);
extern size_t base_video_get_bbbv_solved(void *ptr);
extern size_t base_video_get_ce(void *ptr);
extern double base_video_get_corr(void *ptr);
extern double base_video_get_thrp(void *ptr);
extern double base_video_get_ioe(void *ptr);
extern double base_video_get_path(void *ptr);
extern double base_video_get_stnb(void *ptr);
extern size_t base_video_get_mouse_state(void *ptr);
extern size_t base_video_get_current_event_id(void *ptr);
extern unsigned char base_video_set_current_event_id(void *ptr, size_t id);
extern double base_video_get_current_time(void *ptr);
extern void   base_video_set_current_time(void *ptr, double t);
extern size_t base_video_get_event_count(void *ptr);
extern unsigned char base_video_is_valid(void *ptr);
extern double base_video_event_time(void *ptr, size_t idx);
extern char*  base_video_event_desc(void *ptr, size_t idx);
extern void   base_video_free_event_desc(char *s);
extern struct Board base_video_get_game_board(void *ptr);
extern char*  base_video_get_software(void *ptr);
extern char*  base_video_get_player(void *ptr);
extern void   base_video_free_string(char *s);
extern size_t base_video_get_width(void *ptr);
extern size_t base_video_get_height(void *ptr);
extern size_t base_video_get_mine_num(void *ptr);
extern unsigned short base_video_get_mode(void *ptr);
extern unsigned char  base_video_get_level(void *ptr);
extern unsigned char  base_video_get_nf(void *ptr);
extern unsigned char  base_video_get_is_completed(void *ptr);



static struct Board java2board(JNIEnv *env, jobjectArray jboard) {
    jsize rows = (*env)->GetArrayLength(env, jboard);
    struct Row *rowarr = (struct Row *)malloc(rows * sizeof(struct Row));

    for (jsize i = 0; i < rows; i++) {
        jintArray jrow = (jintArray)(*env)->GetObjectArrayElement(env, jboard, i);
        jsize cols = (*env)->GetArrayLength(env, jrow);
        jint *elems = (*env)->GetIntArrayElements(env, jrow, NULL);

        int32_t *cells = (int32_t *)malloc(cols * sizeof(int32_t));
        for (jsize j = 0; j < cols; j++)
            cells[j] = (int32_t)elems[j];

        (*env)->ReleaseIntArrayElements(env, jrow, elems, JNI_ABORT);
        (*env)->DeleteLocalRef(env, jrow);
        rowarr[i] = (struct Row){ cells, cols };
    }
    return (struct Board){ rowarr, rows };
}

static jobjectArray board2java(JNIEnv *env, struct Board board) {
    jclass intArrCls = (*env)->FindClass(env, "[I");
    jobjectArray result = (*env)->NewObjectArray(env, (jsize)board.n_row, intArrCls, NULL);

    for (size_t i = 0; i < board.n_row; i++) {
        jsize cols = (jsize)board.rows[i].n_column;
        jintArray jrow = (*env)->NewIntArray(env, cols);

        jint *tmp = (jint *)malloc(cols * sizeof(jint));
        for (jsize j = 0; j < cols; j++)
            tmp[j] = (jint)board.rows[i].cells[j];

        (*env)->SetIntArrayRegion(env, jrow, 0, cols, tmp);
        free(tmp);
        (*env)->SetObjectArrayElement(env, result, (jsize)i, jrow);
        (*env)->DeleteLocalRef(env, jrow);
    }
    return result;
}

static jobjectArray boardPoss2java(JNIEnv *env, struct BoardPoss bp) {
    jclass dblArrCls = (*env)->FindClass(env, "[D");
    jobjectArray result = (*env)->NewObjectArray(env, (jsize)bp.n_row, dblArrCls, NULL);

    for (size_t i = 0; i < bp.n_row; i++) {
        jsize cols = (jsize)bp.rows_poss[i].n_column;
        jdoubleArray jrow = (*env)->NewDoubleArray(env, cols);

        jdouble *tmp = (jdouble *)malloc(cols * sizeof(jdouble));
        for (jsize j = 0; j < cols; j++)
            tmp[j] = (jdouble)bp.rows_poss[i].cells_poss[j];

        (*env)->SetDoubleArrayRegion(env, jrow, 0, cols, tmp);
        free(tmp);
        (*env)->SetObjectArrayElement(env, result, (jsize)i, jrow);
        (*env)->DeleteLocalRef(env, jrow);
    }
    return result;
}

JNIEXPORT jint JNICALL Java_ms_1toollib_MsToollib_cal3BV(JNIEnv *env, jclass cls, jobjectArray board) {
    struct Board b = java2board(env, board);
    jint result = (jint)cal_bbbv(b);
    for (size_t i = 0; i < b.n_row; i++) free(b.rows[i].cells);
    free(b.rows);
    return result;
}

JNIEXPORT jint JNICALL Java_ms_1toollib_MsToollib_calZini(JNIEnv *env, jclass cls, jobjectArray board) {
    struct Board b = java2board(env, board);
    jint result = (jint)cal_zini(b);
    for (size_t i = 0; i < b.n_row; i++) free(b.rows[i].cells);
    free(b.rows);
    return result;
}

JNIEXPORT jint JNICALL Java_ms_1toollib_MsToollib_calHzini(JNIEnv *env, jclass cls, jobjectArray board) {
    struct Board b = java2board(env, board);
    jint result = (jint)cal_hzini(b);
    for (size_t i = 0; i < b.n_row; i++) free(b.rows[i].cells);
    free(b.rows);
    return result;
}

JNIEXPORT jint JNICALL Java_ms_1toollib_MsToollib_calRzini(JNIEnv *env, jclass cls, jobjectArray board, jint nIter) {
    struct Board b = java2board(env, board);
    jint result = (jint)cal_rzini(b, (size_t)nIter);
    for (size_t i = 0; i < b.n_row; i++) free(b.rows[i].cells);
    free(b.rows);
    return result;
}

JNIEXPORT jint JNICALL Java_ms_1toollib_MsToollib_calIsl(JNIEnv *env, jclass cls, jobjectArray board) {
    struct Board b = java2board(env, board);
    jint result = (jint)cal_isl(b);
    for (size_t i = 0; i < b.n_row; i++) free(b.rows[i].cells);
    free(b.rows);
    return result;
}

JNIEXPORT jint JNICALL Java_ms_1toollib_MsToollib_calOp(JNIEnv *env, jclass cls, jobjectArray board) {
    struct Board b = java2board(env, board);
    jint result = (jint)cal_op(b);
    for (size_t i = 0; i < b.n_row; i++) free(b.rows[i].cells);
    free(b.rows);
    return result;
}

JNIEXPORT jobjectArray JNICALL Java_ms_1toollib_MsToollib_laymine(JNIEnv *env, jclass cls,
    jint row, jint col, jint mineNum, jint x0, jint y0)
{
    struct Board board = laymine((size_t)row, (size_t)col, (size_t)mineNum, (size_t)x0, (size_t)y0);
    jobjectArray result = board2java(env, board);
    free_board(board);
    return result;
}

JNIEXPORT jobjectArray JNICALL Java_ms_1toollib_MsToollib_calProbabilityOnboard(JNIEnv *env, jclass cls,
    jobjectArray gameBoard, jdouble mineNum)
{
    struct Board gb = java2board(env, gameBoard);
    struct BoardPossReturn pr = cal_probability_onboard(gb, (double)mineNum);
    jobjectArray result = boardPoss2java(env, pr.board_poss);
    free_board_poss(pr);
    for (size_t i = 0; i < gb.n_row; i++) free(gb.rows[i].cells);
    free(gb.rows);
    return result;
}

// ═══════════════════ Video type JNI ═══════════════════

// Helper: jstring → char*
static char* jstr_to_c(JNIEnv *env, jstring js) {
    const char *utf = (*env)->GetStringUTFChars(env, js, NULL);
    char *copy = _strdup(utf);
    (*env)->ReleaseStringUTFChars(env, js, utf);
    return copy;
}

// ─── AvfVideo ───

JNIEXPORT jlong JNICALL Java_ms_1toollib_AvfVideo_nativeNew(JNIEnv *env, jclass cls, jstring fileName) {
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)avf_video_new(fname);
    free(fname);
    return ptr;
}

JNIEXPORT jlong JNICALL Java_ms_1toollib_AvfVideo_nativeNewFromData(JNIEnv *env, jclass cls, jbyteArray data, jstring fileName) {
    jsize len = (*env)->GetArrayLength(env, data);
    jbyte *bytes = (*env)->GetByteArrayElements(env, data, NULL);
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)avf_video_new_from_data((const unsigned char*)bytes, (size_t)len, fname);
    (*env)->ReleaseByteArrayElements(env, data, bytes, JNI_ABORT);
    free(fname);
    return ptr;
}

JNIEXPORT void JNICALL Java_ms_1toollib_AvfVideo_nativeFree(JNIEnv *env, jclass cls, jlong ptr) { avf_video_free((void*)ptr); }

JNIEXPORT jint JNICALL Java_ms_1toollib_AvfVideo_nativeParse(JNIEnv *env, jclass cls, jlong ptr) { return (jint)avf_video_parse((void*)ptr); }

JNIEXPORT jlong JNICALL Java_ms_1toollib_AvfVideo_nativeDataPtr(JNIEnv *env, jclass cls, jlong ptr) { return (jlong)avf_video_data_ptr((void*)ptr); }

// ─── EvfVideo ───

JNIEXPORT jlong JNICALL Java_ms_1toollib_EvfVideo_nativeNew(JNIEnv *env, jclass cls, jstring fileName) {
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)evf_video_new(fname);
    free(fname);
    return ptr;
}

JNIEXPORT jlong JNICALL Java_ms_1toollib_EvfVideo_nativeNewFromData(JNIEnv *env, jclass cls, jbyteArray data, jstring fileName) {
    jsize len = (*env)->GetArrayLength(env, data);
    jbyte *bytes = (*env)->GetByteArrayElements(env, data, NULL);
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)evf_video_new_from_data((const unsigned char*)bytes, (size_t)len, fname);
    (*env)->ReleaseByteArrayElements(env, data, bytes, JNI_ABORT);
    free(fname);
    return ptr;
}

JNIEXPORT void JNICALL Java_ms_1toollib_EvfVideo_nativeFree(JNIEnv *env, jclass cls, jlong ptr) { evf_video_free((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_EvfVideo_nativeParse(JNIEnv *env, jclass cls, jlong ptr) { return (jint)evf_video_parse((void*)ptr); }
JNIEXPORT jlong JNICALL Java_ms_1toollib_EvfVideo_nativeDataPtr(JNIEnv *env, jclass cls, jlong ptr) { return (jlong)evf_video_data_ptr((void*)ptr); }

// ─── MvfVideo ───

JNIEXPORT jlong JNICALL Java_ms_1toollib_MvfVideo_nativeNew(JNIEnv *env, jclass cls, jstring fileName) {
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)mvf_video_new(fname);
    free(fname);
    return ptr;
}

JNIEXPORT jlong JNICALL Java_ms_1toollib_MvfVideo_nativeNewFromData(JNIEnv *env, jclass cls, jbyteArray data, jstring fileName) {
    jsize len = (*env)->GetArrayLength(env, data);
    jbyte *bytes = (*env)->GetByteArrayElements(env, data, NULL);
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)mvf_video_new_from_data((const unsigned char*)bytes, (size_t)len, fname);
    (*env)->ReleaseByteArrayElements(env, data, bytes, JNI_ABORT);
    free(fname);
    return ptr;
}

JNIEXPORT void JNICALL Java_ms_1toollib_MvfVideo_nativeFree(JNIEnv *env, jclass cls, jlong ptr) { mvf_video_free((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_MvfVideo_nativeParse(JNIEnv *env, jclass cls, jlong ptr) { return (jint)mvf_video_parse((void*)ptr); }
JNIEXPORT jlong JNICALL Java_ms_1toollib_MvfVideo_nativeDataPtr(JNIEnv *env, jclass cls, jlong ptr) { return (jlong)mvf_video_data_ptr((void*)ptr); }

// ─── RmvVideo ───

JNIEXPORT jlong JNICALL Java_ms_1toollib_RmvVideo_nativeNew(JNIEnv *env, jclass cls, jstring fileName) {
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)rmv_video_new(fname);
    free(fname);
    return ptr;
}

JNIEXPORT jlong JNICALL Java_ms_1toollib_RmvVideo_nativeNewFromData(JNIEnv *env, jclass cls, jbyteArray data, jstring fileName) {
    jsize len = (*env)->GetArrayLength(env, data);
    jbyte *bytes = (*env)->GetByteArrayElements(env, data, NULL);
    char *fname = jstr_to_c(env, fileName);
    jlong ptr = (jlong)rmv_video_new_from_data((const unsigned char*)bytes, (size_t)len, fname);
    (*env)->ReleaseByteArrayElements(env, data, bytes, JNI_ABORT);
    free(fname);
    return ptr;
}

JNIEXPORT void JNICALL Java_ms_1toollib_RmvVideo_nativeFree(JNIEnv *env, jclass cls, jlong ptr) { rmv_video_free((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_RmvVideo_nativeParse(JNIEnv *env, jclass cls, jlong ptr) { return (jint)rmv_video_parse((void*)ptr); }
JNIEXPORT jlong JNICALL Java_ms_1toollib_RmvVideo_nativeDataPtr(JNIEnv *env, jclass cls, jlong ptr) { return (jlong)rmv_video_data_ptr((void*)ptr); }

// ═══════════════════ BaseVideo JNI (shared) ═══════════════════
// All functions take a data pointer obtained from nativeDataPtr()

JNIEXPORT void JNICALL Java_ms_1toollib_BaseVideo_nativeAnalyse(JNIEnv *env, jclass cls, jlong ptr) { base_video_analyse((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetRtime(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_rtime((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetRtimeMs(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_rtime_ms((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetEtime(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_etime((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetLeft(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_left((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetRight(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_right((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetDouble(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_double((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetCl(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_cl((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetFlag(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_flag((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetBbbvSolved(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_bbbv_solved((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetCe(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_ce((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetCorr(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_corr((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetThrp(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_thrp((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetIoe(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_ioe((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetPath(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_path((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetStnb(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_stnb((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetMouseState(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_mouse_state((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetCurrentEventId(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_current_event_id((void*)ptr); }
JNIEXPORT jbyte JNICALL Java_ms_1toollib_BaseVideo_nativeSetCurrentEventId(JNIEnv *env, jclass cls, jlong ptr, jint id) { return (jbyte)base_video_set_current_event_id((void*)ptr, (size_t)id); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeGetCurrentTime(JNIEnv *env, jclass cls, jlong ptr) { return (jdouble)base_video_get_current_time((void*)ptr); }
JNIEXPORT void JNICALL Java_ms_1toollib_BaseVideo_nativeSetCurrentTime(JNIEnv *env, jclass cls, jlong ptr, jdouble t) { base_video_set_current_time((void*)ptr, (double)t); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetEventCount(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_event_count((void*)ptr); }
JNIEXPORT jbyte JNICALL Java_ms_1toollib_BaseVideo_nativeIsValid(JNIEnv *env, jclass cls, jlong ptr) { return (jbyte)base_video_is_valid((void*)ptr); }
JNIEXPORT jdouble JNICALL Java_ms_1toollib_BaseVideo_nativeEventTime(JNIEnv *env, jclass cls, jlong ptr, jint idx) { return (jdouble)base_video_event_time((void*)ptr, (size_t)idx); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetWidth(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_width((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetHeight(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_height((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetMineNum(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_mine_num((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetMode(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_mode((void*)ptr); }
JNIEXPORT jint JNICALL Java_ms_1toollib_BaseVideo_nativeGetLevel(JNIEnv *env, jclass cls, jlong ptr) { return (jint)base_video_get_level((void*)ptr); }
JNIEXPORT jboolean JNICALL Java_ms_1toollib_BaseVideo_nativeGetNf(JNIEnv *env, jclass cls, jlong ptr) { return base_video_get_nf((void*)ptr) ? JNI_TRUE : JNI_FALSE; }
JNIEXPORT jboolean JNICALL Java_ms_1toollib_BaseVideo_nativeGetIsCompleted(JNIEnv *env, jclass cls, jlong ptr) { return base_video_get_is_completed((void*)ptr) ? JNI_TRUE : JNI_FALSE; }

// String getters: returns C string, Java must call nativeFreeString()
JNIEXPORT jstring JNICALL Java_ms_1toollib_BaseVideo_nativeGetSoftware(JNIEnv *env, jclass cls, jlong ptr) {
    char *s = base_video_get_software((void*)ptr);
    jstring js = (*env)->NewStringUTF(env, s);
    base_video_free_string(s);
    return js;
}

JNIEXPORT jstring JNICALL Java_ms_1toollib_BaseVideo_nativeGetPlayer(JNIEnv *env, jclass cls, jlong ptr) {
    char *s = base_video_get_player((void*)ptr);
    jstring js = (*env)->NewStringUTF(env, s);
    base_video_free_string(s);
    return js;
}

JNIEXPORT jstring JNICALL Java_ms_1toollib_BaseVideo_nativeEventDesc(JNIEnv *env, jclass cls, jlong ptr, jint idx) {
    char *s = base_video_event_desc((void*)ptr, (size_t)idx);
    if (s == NULL) return NULL;
    jstring js = (*env)->NewStringUTF(env, s);
    base_video_free_event_desc(s);
    return js;
}

// getGameBoard: returns int[][]
JNIEXPORT jobjectArray JNICALL Java_ms_1toollib_BaseVideo_nativeGetGameBoard(JNIEnv *env, jclass cls, jlong ptr) {
    struct Board b = base_video_get_game_board((void*)ptr);
    jobjectArray result = board2java(env, b);
    free_board(b);
    return result;
}
