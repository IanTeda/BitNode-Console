//-- ./src/lib/authInterceptor.ts

/**
 * # Auth Interceptor
 */

import logger from "@/lib/logger";
import { UnaryCall, type RpcInterceptor } from "@protobuf-ts/runtime-rpc";

const log = logger.getSubLogger({ name: "AuthInterceptor" });

/**
 * # Auth Interceptor
 *
 * The auth interceptor is responsible for adding the authorization header to the
 * request. It will optionally set the authorisation header in the grpc request if an
 * access token is provided.
 *
 * @param accessToken? - Optional access token to use in the grpc request
 * @returns {RpcInterceptor} - The auth interceptor
 */
const authInterceptor = (accessToken?: string): RpcInterceptor => ({
  interceptUnary(next, method, input, options): UnaryCall {
    // Initialise the meta object if it doesn't exist in the request
    if (!options.meta) {
      options.meta = {};
    }

    // Add Authorization header if access token is provided only
    if (accessToken) {
      options.meta = {
        ...options.meta,
        authorization: `Bearer ${accessToken}`,
      };

      log.debug(
        "Added authorisation bear token to request header: ",
        options.meta,
      );
    }

    return next(method, input, options);
  },
});

export default authInterceptor;
